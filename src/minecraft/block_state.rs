use gfx;
use piston::AssetStore;
use piston::vecmath::vec3_add;
use serialize::json;
use std::cmp::max;
use std::collections::HashMap;
use std::io::fs::File;

use array::*;
use cube;
use chunk::{BiomeId, BlockState, Chunk};
use minecraft::biome::Biomes;
use minecraft::data;
use minecraft::model;
use minecraft::model::{Model, OrthoRotation, Rotate0, Rotate90, Rotate180, Rotate270};
use shader::Vertex;
use texture::{AtlasBuilder, Texture};

pub struct BlockStates {
    models: Vec<ModelAndBehavior>,
    texture: Texture
}

#[deriving(PartialEq, Eq, Clone)]
pub enum RandomOffset {
    NoRandomOffset,
    RandomOffsetXZ,
    RandomOffsetXYZ
}

#[deriving(Clone)]
pub struct ModelAndBehavior {
    pub model: Model,
    pub random_offset: RandomOffset
}

impl ModelAndBehavior {
    pub fn empty() -> ModelAndBehavior {
        ModelAndBehavior {
            model: Model::empty(),
            random_offset: NoRandomOffset
        }
    }

    pub fn is_empty(&self) -> bool {
        self.model.is_empty()
    }
}

impl BlockStates {
    pub fn load<D: gfx::Device>(assets: &AssetStore, d: &mut D) -> BlockStates {

        struct Variant {
            model: String,
            rotate_x: OrthoRotation,
            rotate_y: OrthoRotation,
            uvlock: bool
        }

        let (last_id, _, _) = *data::BLOCK_STATES.last().unwrap();
        let mut models = Vec::with_capacity(last_id as uint + 1);

        let mut atlas = AtlasBuilder::new(assets.path("minecraft/textures").unwrap(), 16, 16);
        let mut partial_model_cache = HashMap::new();
        let mut block_state_cache: HashMap<String, HashMap<String, Variant>> = HashMap::new();
        let variants_str = "variants".to_string();
        let model_str = "model".to_string();

        for &(id, name, variant) in data::BLOCK_STATES.iter() {
            let variants = block_state_cache.find_or_insert_with(name.to_string(), |_| {
                let path = assets.path(format!("minecraft/blockstates/{}.json", name).as_slice());
                match json::from_reader(&mut File::open(&path.unwrap()).unwrap()).unwrap() {
                    json::Object(mut json) => match json.pop(&variants_str).unwrap() {
                        json::Object(variants) => variants.move_iter().map(|(k, v)| {
                            let mut variant = match v {
                                json::Object(o) => o,
                                json::List(l) => {
                                    println!("ignoring {} extra variants for {}#{}",
                                             l.len() - 1, name, k);
                                    match l.move_iter().next() {
                                        Some(json::Object(o)) => Some(o),
                                        _ => None
                                    }.unwrap()
                                }
                                json => fail!("{}#{} has invalid value {}", name, k, json)
                            };
                            let model = match variant.pop(&model_str).unwrap() {
                                json::String(s) => s,
                                json => fail!("'model' has invalid value {}", json)
                            };
                            let rotate_x = variant.find_with(|k| "x".cmp(&k.as_slice())).map_or(Rotate0, |r| {
                                match OrthoRotation::from_json(r) {
                                    Some(r) => r,
                                    None => fail!("invalid rotation for x {}", r)
                                }
                            });
                            let rotate_y = variant.find_with(|k| "y".cmp(&k.as_slice())).map_or(Rotate0, |r| {
                                match OrthoRotation::from_json(r) {
                                    Some(r) => r,
                                    None => fail!("invalid rotation for y {}", r)
                                }
                            });
                            match variant.find_with(|k| "z".cmp(&k.as_slice())) {
                                Some(r) => println!("ignoring z rotation {} in {}", r, name),
                                None => {}
                            }
                            let uvlock = variant.find_with(|k| "uvlock".cmp(&k.as_slice()))
                                                .map_or(false, |x| x.as_boolean().unwrap());
                            (k, Variant {
                                model: model,
                                rotate_x: rotate_x,
                                rotate_y: rotate_y,
                                uvlock: uvlock
                            })
                        }).collect(),
                        json => fail!("'variants' has invalid value {}", json)
                    },
                    json => fail!("root object has invalid value {}", json)
                }
            });

            // Special case some kinds of blocks for the default render state.
            let variant = if variant.ends_with(",shape=outer_right") {
                variants.find(&format!("{}=straight", variant.slice_to(variant.len() - 12)))
            } else {
                variants.find_equiv(&variant)
            }.unwrap();
            let mut model = Model::load(variant.model.as_slice(), assets,
                                        &mut atlas, &mut partial_model_cache);

            let rotate_faces = |m: &mut Model, ix, iy, rot_mat: [i32, ..4]| {
                let [a, b, c, d] = rot_mat.map(|x: i32| x as f32);
                for face in m.faces.mut_iter() {
                    for vertex in face.vertices.mut_iter() {
                        let xyz = &mut vertex.xyz;
                        let [x, y] = [ix, iy].map(|i| xyz[i] - 0.5);
                        xyz[ix] = a * x + b * y + 0.5;
                        xyz[iy] = c * x + d * y + 0.5;
                    }
                    face.cull_face.mutate(|f| {
                        let [a, b, c, d] = rot_mat;
                        let mut dir = f.direction();
                        let [x, y] = [dir[ix], dir[iy]];
                        dir[ix] = a * x + b * y;
                        dir[iy] = c * x + d * y;
                        cube::Face::from_direction(dir).unwrap()
                    });
                    if variant.uvlock {
                        // Skip over faces that are constant in the ix or iy axis.
                        let xs = face.vertices.map(|v| v.xyz[ix]);
                        if xs.map(|x| x == xs[0]) == [true, true, true, true] {
                            continue;
                        }
                        let ys = face.vertices.map(|v| v.xyz[iy]);
                        if ys.map(|y| y == ys[0]) == [true, true, true, true] {
                            continue;
                        }

                        let uvs = face.vertices.map(|x| x.uv);
                        let uv_min = [0, 1].map(|i| (uvs[0][i]).min(uvs[1][i])
                                                .min(uvs[2][i]).min(uvs[3][i]));
                        let [u_base, v_base] = uv_min.map(|x| (x / 16.0).floor() * 16.0);
                        for vertex in face.vertices.mut_iter() {
                            let uv = &mut vertex.uv;
                            let [u, v] = [uv[0] - u_base, uv[1] - v_base].map(|x| x - 8.0);
                            uv[0] = a * u - b * v + 8.0 + u_base;
                            uv[1] =-c * u + d * v + 8.0 + v_base;
                        }
                    }
                }
            };
            let rotate_faces = |m: &mut Model, ix, iy, r: OrthoRotation| {
                match r {
                    Rotate0 => {}
                    Rotate90 => rotate_faces(m, ix, iy, [0,-1,
                                                         1, 0]),
                    Rotate180 => rotate_faces(m, ix, iy, [-1, 0,
                                                           0,-1]),
                    Rotate270 => rotate_faces(m, ix, iy, [0, 1,
                                                         -1, 0]),
                }
            };

            rotate_faces(&mut model, 2, 1, variant.rotate_x);
            rotate_faces(&mut model, 0, 2, variant.rotate_y);

            models.grow_set(id as uint, &ModelAndBehavior::empty(), ModelAndBehavior {
                model: model,
                random_offset: NoRandomOffset
            });
        }

        drop(partial_model_cache);
        drop(block_state_cache);

        let texture = atlas.complete(d);
        let u_unit = 1.0 / (texture.width as f32);
        let v_unit = 1.0 / (texture.height as f32);

        for &(id, _, _) in data::BLOCK_STATES.iter() {
            for face in models.get_mut(id as uint).model.faces.mut_iter() {
                for vertex in face.vertices.mut_iter() {
                    vertex.uv[0] *= u_unit;
                    vertex.uv[1] *= v_unit;
                }
            }
        }

        // Patch some models.
        for &(id, name, _) in data::BLOCK_STATES.iter() {
            if name == "tall_grass" {
                // Add a random offset to dead_bush, tall_grass and fern.
                for &id in [id - 1, id, id + 2].iter() {
                    models.get_mut(id as uint).random_offset = RandomOffsetXYZ;
                }
            }
        }

        BlockStates {
            models: models,
            texture: texture
        }
    }

    pub fn get_model<'a>(&'a self, i: BlockState) -> Option<&'a ModelAndBehavior> {
        let i = i.value as uint;
        if i >= self.models.len() || self.models[i].is_empty() {
            None
        } else {
            Some(&self.models[i])
        }
    }

    pub fn texture<'a>(&'a self) -> &'a Texture {
        &self.texture
    }

    pub fn is_opaque(&self, i: BlockState) -> bool {
        let i = i.value as uint;
        if i >= self.models.len() {
            false
        } else {
            self.models[i].model.opaque
        }
    }
}

pub fn fill_buffer(block_states: &BlockStates, biomes: &Biomes, buffer: &mut Vec<Vertex>,
                   coords: [i32, ..3], chunks: [[[&Chunk, ..3], ..3], ..3],
                   column_biomes: [[Option<&[[BiomeId, ..16], ..16]>, ..3], ..3]) {
    let chunk_xyz = coords.map(|x| x as f32 * 16.0);
    for y in range(0, 16) {
        for z in range(0, 16) {
            for x in range(0, 16) {
                let at = |dx: i32, dy: i32, dz: i32| {
                    let [x, y, z] = [x + dx as uint, y + dy as uint, z + dz as uint].map(|x| x + 16);
                    let chunk = chunks[y / 16][z / 16][x / 16];
                    let [x, y, z] = [x, y, z].map(|x| x % 16);
                    (chunk.blocks[y][z][x], chunk.light_levels[y][z][x])
                };
                let model = match block_states.get_model(at(0, 0, 0).val0()) {
                    Some(model) => model,
                    None => continue
                };
                let block_xyz = vec3_add([x, y, z].map(|x| x as f32), chunk_xyz);
                let block_xyz = match model.random_offset {
                    NoRandomOffset => block_xyz,
                    random_offset => {
                        let [x, _, z] = block_xyz;
                        let seed = (x as i32 * 3129871) as i64 ^ (z as i64) * 116129781;
                        let value = seed * seed * 42317861 + seed * 11;
                        let ox = (((value >> 16) & 15) as f32 / 15.0 - 0.5) * 0.5;
                        let oz = (((value >> 24) & 15) as f32 / 15.0 - 0.5) * 0.5;
                        let oy = if random_offset == RandomOffsetXYZ {
                            (((value >> 20) & 15) as f32 / 15.0 - 1.0) * 0.2
                        } else { 0.0 };
                        vec3_add(block_xyz, [ox, oy, oz])
                    }
                };
                let model = &model.model;
                for face in model.faces.iter() {
                    match face.cull_face {
                        Some(cull_face) => {
                            let [dx, dy, dz] = cull_face.direction();
                            if block_states.is_opaque(at(dx, dy, dz).val0()) {
                                continue;
                            }
                        }
                        None => {}
                    }

                    let tint_source = if face.tint {
                        model.tint_source
                    } else {
                        model::NoTint
                    };

                    let v = face.vertices.map(|vertex| {
                        // Average tint and light around the vertex.
                        let (rgb, mut num_colors) = match tint_source {
                            model::NoTint => ([0xff, 0xff, 0xff], 1.0),
                            model::GrassTint | model::FoliageTint => ([0x00, 0x00, 0x00], 0.0),
                            model::RedstoneTint => ([0xff, 0x00, 0x00], 1.0)
                        };
                        let mut rgb = rgb.map(|x: u8| x as f32 / 255.0);
                        let (mut sum_light_level, mut num_light_level) = (0.0, 0.0);

                        let [dx, dy, dz] = vertex.xyz.map(|x| x.round() as i32);
                        for &dx in [dx - 1, dx].iter() {
                            for &dz in [dz - 1, dz].iter() {
                                for &dy in [dy - 1, dy].iter() {
                                    let (neighbor, light_level) = at(dx, dy, dz);
                                    if block_states.is_opaque(neighbor) {
                                        continue;
                                    }
                                    let light_level = max(light_level.block_light(), light_level.sky_light());
                                    sum_light_level += light_level as f32;
                                    num_light_level += 1.0;
                                }
                                match tint_source {
                                    model::NoTint | model::RedstoneTint => continue,
                                    model::GrassTint | model::FoliageTint => {}
                                }
                                let [x, z] = [x + dx as uint, z + dz as uint].map(|x| x + 16);
                                let biome = match column_biomes[z / 16][x / 16] {
                                    Some(biome) => biomes[biome[z % 16][x % 16]],
                                    None => continue
                                };
                                rgb = vec3_add(rgb, match tint_source {
                                    model::NoTint | model::RedstoneTint => continue,
                                    model::GrassTint => biome.grass_color,
                                    model::FoliageTint => biome.foliage_color,
                                }.map(|x| x as f32 / 255.0));
                                num_colors += 1.0;
                            }
                        }

                        let light_factor = 0.5 + if num_light_level != 0.0 {
                            sum_light_level / num_light_level / 15.0 / 2.0
                        } else { 0.0 };

                        Vertex {
                            xyz: vec3_add(block_xyz, vertex.xyz),
                            uv: vertex.uv,
                            rgb: rgb.map(|x| x * light_factor / num_colors)
                        }
                    });

                    // Split the clockwise quad into two clockwise triangles.
                    buffer.push_all([v[0], v[1], v[2]]);
                    buffer.push_all([v[2], v[3], v[0]]);
                }
            }
        }
    }
}
