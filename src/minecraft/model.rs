use std::collections::HashMap;
use std::collections::hash_map::Entry::{ Occupied, Vacant };
use std::f32::consts::{PI, SQRT_2};
use std::f32::INFINITY;
use std::fs::File;
use std::path::Path;
use std::str::FromStr;

use self::OrthoRotation::*;

use array::*;
use cube;
use serialize::json;
use gfx_voxel::texture::AtlasBuilder;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub xyz: [f32; 3],
    pub uv: [f32; 2]
}

#[derive(Copy, Clone)]
pub enum Tint {
    None,
    Grass,
    Foliage,
    Redstone
}

#[derive(Copy, Clone)]
pub enum OrthoRotation {
    Rotate0,
    Rotate90,
    Rotate180,
    Rotate270
}

impl OrthoRotation {
    pub fn from_json(json: &json::Json) -> Option<OrthoRotation> {
        json.as_i64().and_then(|r| Some(match r {
            0 => Rotate0,
            90 => Rotate90,
            180 => Rotate180,
            270 => Rotate270,
            _ => return None
        }))
    }
}

#[derive(Copy)]
pub struct Face {
    pub vertices: [Vertex; 4],
    pub tint: bool,
    pub cull_face: Option<cube::Face>,
    pub ao_face: Option<cube::Face>
}

impl Clone for Face {
    fn clone(&self) -> Face { *self }
}

#[derive(Clone)]
enum PartialTexture {
    Variable(String),
    Coords(f32, f32)
}

#[derive(Clone)]
pub struct PartialModel {
    textures: HashMap<String, PartialTexture>,
    faces: Vec<(Face, String)>,
    full_faces: Vec<usize>,
    no_ambient_occlusion: bool
}

#[derive(Copy, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Opacity {
    Transparent,
    TranslucentSolid,
    TransparentSolid,
    Opaque
}

impl Opacity {
    pub fn is_opaque(self) -> bool {
        self == Opacity::Opaque
    }

    pub fn is_solid(self) -> bool {
        self != Opacity::Transparent
    }
}

#[derive(Clone)]
pub struct Model {
    pub faces: Vec<Face>,
    pub opacity: Opacity,
    pub tint_source: Tint
}

fn array3_num<T, F>(json: &json::Json, mut f: F) -> [T; 3] where F: FnMut(f64) -> T {
    Array::from_iter(json.as_array().unwrap().iter().map(|x| f(x.as_f64().unwrap())))
}

fn clone_parent(m: &PartialModel, _a: &mut AtlasBuilder) -> PartialModel {
    m.clone()
}

impl PartialModel {
    fn load<T, F>(name: &str, assets: &Path, atlas: &mut AtlasBuilder,
               cache: &mut HashMap<String, PartialModel>,
               mut f: F) -> T
        where F: FnMut(&PartialModel, &mut AtlasBuilder) -> T
    {
        match cache.get(name) {
            Some(model) => return f(model, atlas),
            None => {}
        }
        let path = assets.join(Path::new(format!("minecraft/models/{}.json", name).as_str()));
        let obj = json::Json::from_reader(&mut File::open(&path).unwrap()).unwrap();

        let mut model = match obj.find("parent").and_then(|x| x.as_string()) {
            // FIXME(toqueteos): Cthulu himself came here and inspired me, if we use a closure here instead of
            // "clone_parent" this would trigger an error: "reached the recursion limit during monomorphization"
            Some(parent) => PartialModel::load(parent, assets, atlas, cache, clone_parent),
            None => PartialModel {
                textures: HashMap::new(),
                faces: vec![],
                full_faces: vec![],
                no_ambient_occlusion: false
            }
        };

        match obj.find("ambientocclusion").and_then(|x| x.as_boolean()) {
            Some(ambient_occlusion) => model.no_ambient_occlusion = !ambient_occlusion,
            None => {}
        }

        match obj.find("textures").and_then(|x| x.as_object()) {
            Some(textures) => for (name, tex) in textures.iter() {
                let tex = tex.as_string().unwrap();
                let tex = if tex.starts_with("#") {
                    PartialTexture::Variable(tex[1..].to_string())
                } else {
                    let (u, v) = atlas.load(tex);
                    PartialTexture::Coords(u as f32, v as f32)
                };
                model.textures.insert(name.clone(), tex);
            },
            None => {}
        }

        match obj.find("elements").and_then(|x: &json::Json| x.as_array().map(|x| x.clone())) {
            Some(elements) => for element in elements.iter().map(|x| x) {
                let from = array3_num(element.find("from").unwrap(), |x| x as f32 / 16.0);
                let to = array3_num(element.find("to").unwrap(), |x| x as f32 / 16.0);
                let scale = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];

                let is_full_cube = from == [0.0, 0.0, 0.0] && to == [1.0, 1.0, 1.0];
                let element_start = model.faces.len();

                for (k, v) in element.find("faces").unwrap().as_object().unwrap().iter() {
                    let face: cube::Face = FromStr::from_str(k.as_str()).unwrap();
                    let [u0, v0, u1, v1] = match v.find("uv") {
                        Some(uv) => {
                            Array::from_iter(uv.as_array().unwrap().iter().map(|x| x.as_f64().unwrap() as f32))
                        }
                        None => match face {
                            cube::West  | cube::East  => [from[2], from[1], to[2], to[1]],
                            cube::Down  | cube::Up    => [from[0], from[2], to[0], to[2]],
                            cube::North | cube::South => [from[0], from[1], to[0], to[1]]
                        }.map(|x| x * 16.0)
                    };

                    let tex = v.find("texture").unwrap().as_string().unwrap();
                    assert!(tex.starts_with("#"));
                    let tex = tex[1..].to_string();

                    let cull_face = v.find("cullface").map(|s| {
                        FromStr::from_str(s.as_string().unwrap()).unwrap()
                    });

                    if cull_face.is_some() && cull_face != Some(face) {
                        println!("odd case: cull_face = {:?} for face = {:?}", cull_face.unwrap(), face);
                    }

                    let tint = v.find("tintindex").map(|x| {
                        let x = x.as_i64().unwrap();
                        if x != 0 {
                            println!("odd case: tint_index = {}", x);
                        }
                    }).is_some();

                    if cull_face == Some(face) && is_full_cube {
                        model.full_faces.push(model.faces.len());
                    }

                    let rotation = v.find("rotation").map_or(Rotate0, |r| {
                        match OrthoRotation::from_json(r) {
                            Some(r) => r,
                            None => panic!("invalid rotation for face {}", r)
                        }
                    });

                    let xyz = face.vertices(from, scale);
                    // Swap vertical texture coordinates.
                    let [v0, v1] = [v1, v0];
                    // Bring texture coordinates closer to avoid seams.
                    let u_center = (u0 + u1) / 2.0;
                    let [u0, u1] = [u0, u1].map(|u| u - (u - u_center).signum() / 128.0);
                    let v_center = (v0 + v1) / 2.0;
                    let [v0, v1] = [v0, v1].map(|v| v - (v - v_center).signum() / 128.0);
                    // Clockwise quad (from bottom-right to top-right).
                    let uvs = [
                        [u1, v0],
                        [u0, v0],
                        [u0, v1],
                        [u1, v1]
                    ].map(|[u, v]| match rotation {
                        Rotate0 => [u, v],
                        Rotate90 => [v, 16.0 - u],
                        Rotate180 => [16.0 - u, 16.0 - v],
                        Rotate270 => [16.0 - v, u]
                    });

                    model.faces.push((Face {
                        vertices: Array::from_fn(|i| Vertex { xyz: xyz[i], uv: uvs[i] }),
                        tint: tint,
                        cull_face: cull_face,
                        ao_face: Some(face)
                    }, tex));
                }

                match element.find("rotation") {
                    Some(r) => {
                        let angle = r.find("angle").unwrap().as_f64().unwrap();
                        let angle = angle as f32 / 180.0 * PI;
                        let rescale = r.find("rescale").map_or(false, |x| x.as_boolean().unwrap());
                        let origin = array3_num(r.find("origin").unwrap(), |x| x as f32 / 16.0);

                        let (s, c) = (angle.sin(), angle.cos());
                        let mut rot = |ix: usize, iy: usize| {
                            for &mut (ref mut face, _) in model.faces[element_start..].iter_mut() {
                                face.ao_face = None;

                                let [ox, oy] = [origin[ix], origin[iy]];
                                for v in face.vertices.iter_mut() {
                                    let [x, y] = [v.xyz[ix] - ox, v.xyz[iy] - oy];
                                    v.xyz[ix] = x * c + y * s;
                                    v.xyz[iy] =-x * s + y * c;
                                }

                                if rescale {
                                    for v in face.vertices.iter_mut() {
                                        v.xyz[ix] *= SQRT_2;
                                        v.xyz[iy] *= SQRT_2;
                                    }
                                }

                                for v in face.vertices.iter_mut() {
                                    v.xyz[ix] += ox;
                                    v.xyz[iy] += oy;
                                }
                            }
                        };
                        match r.find("axis").unwrap().as_string().unwrap() {
                            "x" => rot(2, 1),
                            "y" => rot(0, 2),
                            "z" => rot(1, 0),
                            axis => panic!("invalid rotation axis {}", axis)
                        }
                    }
                    None => {}
                }
            },
            None => {}
        }

        match cache.entry(name.to_string()) {
            Occupied(entry) => f(entry.get(), atlas),
            Vacant(entry) => f(entry.insert(model), atlas)
        }
    }
}

impl Model {
    pub fn load(name: &str, assets: &Path, atlas: &mut AtlasBuilder,
                cache: &mut HashMap<String, PartialModel>) -> Model {
        PartialModel::load(format!("block/{}", name).as_str(), assets, atlas, cache, |partial, atlas| {
            let mut faces: Vec<Face> = partial.faces.iter().map(|&(mut face, ref tex)| {
                fn texture_coords(textures: &HashMap<String, PartialTexture>,
                                  tex: &String) -> Option<(f32, f32)> {
                    match textures.get(tex) {
                        Some(&PartialTexture::Variable(ref tex)) => texture_coords(textures, tex),
                        Some(&PartialTexture::Coords(u, v)) => Some((u, v)),
                        None => None
                    }
                }
                let (u, v) = texture_coords(&partial.textures, tex).unwrap();
                for vertex in face.vertices.iter_mut() {
                    vertex.uv[0] += u;
                    vertex.uv[1] += v;
                }
                face
            }).collect();

            let mut full_faces = [Opacity::Transparent; 6];
            if partial.full_faces.len() >= 6 {
                for &i in partial.full_faces.iter() {
                    let face = faces[i].cull_face.unwrap() as usize;
                    if full_faces[face] == Opacity::Opaque {
                        continue;
                    }
                    let (mut min_u, mut min_v) = (INFINITY, INFINITY);
                    let (mut max_u, mut max_v) = (0.0, 0.0);
                    for vertex in faces[i].vertices.iter() {
                        let [u, v] = vertex.uv;
                        min_u = u.min(min_u);
                        min_v = v.min(min_v);
                        max_u = u.max(max_u);
                        max_v = v.max(max_v);
                    }
                    let (u0, v0) = (min_u.floor() as u32, min_v.floor() as u32);
                    let (u1, v1) = (max_u.ceil() as u32, max_v.ceil() as u32);
                    let rect = [u0, v0, u1 - u0, v1 - v0];
                    let opacity = match atlas.min_alpha(rect) {
                        0 => Opacity::TransparentSolid,
                        255 => Opacity::Opaque,
                        _ => Opacity::TranslucentSolid
                    };
                    if full_faces[face] < opacity {
                        full_faces[face] = opacity;
                    }
                }
            }

            if !partial.no_ambient_occlusion {
                if faces.iter().any(|f| f.ao_face.is_none()) {
                    println!("Warning: model {} uses AO but has faces which are unsuitable", name);
                }
            } else {
                for face in faces.iter_mut() {
                    face.ao_face = None;
                }
            }

            let tint_source = if faces.iter().any(|f| f.tint) {
                match name {
                    name if name.starts_with("grass_") ||
                            name.starts_with("double_grass_") ||
                            name.starts_with("double_fern_") => Tint::Grass,
                    "reeds" | "fern" | "tall_grass" => Tint::Grass,
                    name if name.ends_with("_leaves") || name.ends_with("_stem_fruit") ||
                            name.starts_with("vine_") || name.starts_with("stem_") => Tint::Foliage,
                    "waterlily" => Tint::Foliage,
                    name if name.starts_with("redstone_") => Tint::Redstone,
                    _ => {
                        println!("tint source not known for '{}'", name);
                        Tint::None
                    }
                }
            } else {
                Tint::None
            };

            Model {
                faces: faces,
                opacity: *full_faces.iter().min().unwrap(),
                tint_source: tint_source
            }
        })
    }

    pub fn empty() -> Model {
        Model {
            faces: Vec::new(),
            opacity: Opacity::Transparent,
            tint_source: Tint::None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.faces.is_empty()
    }
}
