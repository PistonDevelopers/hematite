use gfx;
use piston::AssetStore;
use serialize::json;
use std::collections::HashMap;
use std::io::fs::File;

use array::*;
use cube;
use chunk::BlockState;
use minecraft::data;
use minecraft::model::{Model, OrthoRotation, Rotate0, Rotate90, Rotate180, Rotate270};
use texture::{AtlasBuilder, Texture};

pub struct BlockStates {
    models: Vec<Model>,
    texture: Texture
}

struct BlockStateVariant {
    model: String,
    rotate_x: OrthoRotation,
    rotate_y: OrthoRotation,
    uvlock: bool
}

impl BlockStates {
    pub fn load<D: gfx::Device>(assets: &AssetStore, d: &mut D) -> BlockStates {
        let (last_id, _, _) = *data::BLOCK_STATES.last().unwrap();
        let mut models = Vec::with_capacity(last_id as uint + 1);

        let mut atlas = AtlasBuilder::new(assets.path("minecraft/textures").unwrap(), 16, 16);
        let mut partial_model_cache = HashMap::new();
        let mut block_state_cache: HashMap<String, HashMap<String, BlockStateVariant>> = HashMap::new();
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
                            (k, BlockStateVariant {
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

            models.grow_set(id as uint, &Model::empty(), model);
        }

        drop(partial_model_cache);
        drop(block_state_cache);

        let texture = atlas.complete(d);
        let u_unit = 1.0 / (texture.width as f32);
        let v_unit = 1.0 / (texture.height as f32);

        for &(id, _, _) in data::BLOCK_STATES.iter() {
            for face in models.get_mut(id as uint).faces.mut_iter() {
                for vertex in face.vertices.mut_iter() {
                    vertex.uv[0] *= u_unit;
                    vertex.uv[1] *= v_unit;
                }
            }
        }

        BlockStates {
            models: models,
            texture: texture
        }
    }

    pub fn get_model<'a>(&'a self, i: BlockState) -> Option<&'a Model> {
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
            self.models[i].opaque
        }
    }
}
