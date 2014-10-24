use device::draw::CommandBuffer;
use gfx::Device;
use vecmath::vec3_add;
use serialize::json;
use std::cmp::max;
use std::collections::HashMap;
use std::io::fs::File;
use std::num::next_power_of_two;
use std::str::{Owned, SendStr, Slice};
use std::collections::hashmap::{ Occupied, Vacant };

use array::*;
use cube;
use chunk::{BiomeId, BlockState, Chunk};
use minecraft::biome::Biomes;
use minecraft::data::BLOCK_STATES;
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

#[deriving(PartialEq, Eq, Clone)]
pub enum Direction {
    DirDown,
    DirUp,
    DirNorth,
    DirSouth,
    DirWest,
    DirEast,

    // Some diagonal directions (used by redstone).
    DirUpNorth,
    DirUpSouth,
    DirUpWest,
    DirUpEast
}

impl Direction {
    pub fn xyz(self) -> [i32, ..3] {
        match self {
            DirDown => [0, -1, 0],
            DirUp => [0, 1, 0],
            DirNorth => [0, 0, -1],
            DirSouth => [0, 0, 1],
            DirWest => [-1, 0, 0],
            DirEast => [1, 0, 0],

            DirUpNorth => [0, 1, -1],
            DirUpSouth => [0, 1, 1],
            DirUpWest => [-1, 1, 0],
            DirUpEast => [1, 1, 0]
        }
    }
}

#[deriving(Clone)]
pub enum PolymorphDecision {
    // Stop and use this block state ID for the model.
    PickBlockState(u16),

    // Each of these checks a condition and continues if true,
    // or jumps to the provided u8 'else' index otherwise.
    // Blocks are specified with a signed offset from the block itself.
    // The 'OrSolid' variants also check for any solid blocks.
    IfBlock(Direction, i8, u8),
    IfBlockOrSolid(Direction, i8, u8),
    //IfGroup(Direction, Group, u8),
    //IfGroupOrSolid(Direction, Group, u8)
}

struct Description {
    id: u16,
    name: &'static str,
    variant: SendStr,
    random_offset: RandomOffset,
    polymorph_oracle: Vec<PolymorphDecision>
}

#[deriving(Clone)]
pub struct ModelAndBehavior {
    pub model: Model,
    pub random_offset: RandomOffset,
    pub polymorph_oracle: Vec<PolymorphDecision>
}

impl ModelAndBehavior {
    pub fn empty() -> ModelAndBehavior {
        ModelAndBehavior {
            model: Model::empty(),
            random_offset: NoRandomOffset,
            polymorph_oracle: vec![]
        }
    }

    pub fn is_empty(&self) -> bool {
        self.model.is_empty()
    }
}

impl BlockStates {
    pub fn load<D: Device<C>, C: CommandBuffer>(
        assets: &Path, d: &mut D
    ) -> BlockStates {
        let mut last_id = BLOCK_STATES.last().map_or(0, |state| state.val0());
        let mut states = Vec::<Description>::with_capacity(next_power_of_two(BLOCK_STATES.len()));
        let mut extras = vec![];
        let mut flower1 = None::<u16>;
        let mut flower2 = None::<u16>;
        for (i, &(id, name, variant)) in BLOCK_STATES.iter().enumerate() {
            let mut polymorph_oracle = vec![];
            let mut random_offset = NoRandomOffset;

            // Find double_plant.
            if variant == "half=upper" {
                if name != "paeonia" {
                    println!("Warning: unknown upper double_plant {}", name);
                }
                let (_, lower_name, lower_variant) = BLOCK_STATES[i - 1];
                assert!(lower_name == name && lower_variant == "half=lower");
                let lower = BLOCK_STATES.slice_to(i - 1).iter().enumerate().rev();
                let mut lower = lower.take_while(|&(i, &(id, _, variant))| {
                    id + 1 == BLOCK_STATES[i + 1].val0() && variant == "half=lower"
                });
                // Note: excluding paeonia itself, which works as-is.
                let num_plants = lower.count();

                for j in range(i - 1 - num_plants, i - 1) {
                    last_id += 1;
                    let (_, lower_name, _) = BLOCK_STATES[j];
                    extras.push(Description {
                        id: last_id,
                        name: lower_name,
                        variant: Slice("half=upper"),
                        random_offset: RandomOffsetXZ,
                        polymorph_oracle: vec![]
                    });
                    states.get_mut(j).random_offset = RandomOffsetXZ;

                    let next_index = polymorph_oracle.len() as u8;
                    polymorph_oracle.push_all([
                        IfBlock(DirDown, (BLOCK_STATES[j].val0() - id) as i8, next_index + 2),
                        PickBlockState(last_id)
                    ]);
                }
                random_offset = RandomOffsetXZ;
                polymorph_oracle.push(PickBlockState(id));
            }

            if name == "dandelion" {
                flower1 = Some(id);
            } else if name == "poppy" {
                flower2 = Some(id);
            } else if ["dead_bush", "tall_grass", "fern"].contains(&name) {
                random_offset = RandomOffsetXYZ;
            }

            if flower1 == Some(id & !0xf) || flower2 == Some(id & !0xf) {
                random_offset = RandomOffsetXZ;
            }

            let variant = if variant.ends_with(",shape=outer_right") {
                Owned(format!("{}=straight", variant.slice_to(variant.len() - 12)))
            } else {
                Slice(variant)
            };

            states.push(Description {
                id: id,
                name: name,
                variant: variant,
                random_offset: random_offset,
                polymorph_oracle: polymorph_oracle
            });
        }
        states.extend(extras.into_iter());

        BlockStates::load_with_states(assets, d, states)
    }

    fn load_with_states<D: Device<C>, C: CommandBuffer>(
        assets: &Path, d: &mut D,
        states: Vec<Description>
    ) -> BlockStates {
        struct Variant {
            model: String,
            rotate_x: OrthoRotation,
            rotate_y: OrthoRotation,
            uvlock: bool
        }

        let last_id = states.last().map_or(0, |state| state.id);
        let mut models = Vec::with_capacity(last_id as uint + 1);

        let mut atlas = AtlasBuilder::new(assets.join(Path::new("minecraft/textures")), 16, 16);
        let mut partial_model_cache = HashMap::new();
        let mut block_state_cache: HashMap<String, HashMap<String, Variant>> = HashMap::new();
        let variants_str = "variants".to_string();
        let model_str = "model".to_string();

        for state in states.into_iter() {
            let variants = match block_state_cache.entry(state.name.to_string()) {
                Occupied(entry) => entry.into_mut(),
                Vacant(entry) => entry.set({
                    let name = state.name;
                    let path = assets.join(Path::new(format!("minecraft/blockstates/{}.json", name).as_slice()));
                    match json::from_reader(&mut File::open(&path).unwrap()).unwrap() {
                        json::Object(mut json) => match json.pop(&variants_str).unwrap() {
                            json::Object(variants) => variants.into_iter().map(|(k, v)| {
                                let mut variant = match v {
                                    json::Object(o) => o,
                                    json::List(l) => {
                                        println!("ignoring {} extra variants for {}#{}",
                                                 l.len() - 1, name, k);
                                        match l.into_iter().next() {
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
                })
            };

            let variant = match state.variant {
                Owned(ref variant) => variants.find(variant),
                Slice(ref variant) => variants.find_equiv(variant)
            }.unwrap();
            let mut model = Model::load(variant.model.as_slice(), assets,
                                        &mut atlas, &mut partial_model_cache);

            let rotate_faces = |m: &mut Model, ix, iy, rot_mat: [i32, ..4]| {
                let [a, b, c, d] = rot_mat.map(|x: i32| x as f32);
                for face in m.faces.iter_mut() {
                    for vertex in face.vertices.iter_mut() {
                        let xyz = &mut vertex.xyz;
                        let [x, y] = [ix, iy].map(|i| xyz[i] - 0.5);
                        xyz[ix] = a * x + b * y + 0.5;
                        xyz[iy] = c * x + d * y + 0.5;
                    }
                    let fixup_cube_face = |f: cube::Face| {
                        let [a, b, c, d] = rot_mat;
                        let mut dir = f.direction();
                        let [x, y] = [dir[ix], dir[iy]];
                        dir[ix] = a * x + b * y;
                        dir[iy] = c * x + d * y;
                        cube::Face::from_direction(dir).unwrap()
                    };
                    face.cull_face = match face.cull_face {
                        None => None,
                        Some(f) => Some(fixup_cube_face(f))
                    };
                    face.ao_face = match face.ao_face {
                        None => None,
                        Some(f) => Some(fixup_cube_face(f))
                    };
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
                        for vertex in face.vertices.iter_mut() {
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

            let len = models.len();
            if state.id as uint >= len {
                models.grow(state.id as uint - len + 1, ModelAndBehavior::empty());
            }
            *models.get_mut(state.id as uint) = ModelAndBehavior {
                model: model,
                random_offset: state.random_offset,
                polymorph_oracle: state.polymorph_oracle
            };
        }

        drop(partial_model_cache);
        drop(block_state_cache);

        let texture = atlas.complete(d);
        let u_unit = 1.0 / (texture.width as f32);
        let v_unit = 1.0 / (texture.height as f32);

        for model in models.iter_mut() {
            for face in model.model.faces.iter_mut() {
                for vertex in face.vertices.iter_mut() {
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

    pub fn get_opacity(&self, i: BlockState) -> model::Opacity {
        let i = i.value as uint;
        if i >= self.models.len() {
            model::Transparent
        } else {
            self.models[i].model.opacity
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
                let at = |dir: [i32, ..3]| {
                    let [dx, dy, dz] = dir.map(|x| x as uint);
                    let [x, y, z] = [x + dx, y + dy, z + dz].map(|x| x + 16);
                    let chunk = chunks[y / 16][z / 16][x / 16];
                    let [x, y, z] = [x, y, z].map(|x| x % 16);
                    (chunk.blocks[y][z][x], chunk.light_levels[y][z][x])
                };
                let this_block = at([0, 0, 0]).val0();
                let model = match block_states.get_model(this_block) {
                    Some(model) if !model.polymorph_oracle.is_empty() => {
                        let mut i = 0;
                        let result;
                        loop {
                            let (cond, idx) = match model.polymorph_oracle[i] {
                                PickBlockState(id) => {
                                    result = &block_states.models[id as uint];
                                    break;
                                }
                                IfBlock(dir, offset, idx) => {
                                    let id = this_block.value + offset as u16;
                                    (at(dir.xyz()).val0().value == id, idx)
                                }
                                IfBlockOrSolid(dir, offset, idx) => {
                                    let id = this_block.value + offset as u16;
                                    let other = at(dir.xyz()).val0();
                                    (other.value == id ||
                                     block_states.get_opacity(other).is_opaque(), idx)
                                }
                                /*IfGroup(dir, group, idx) => {
                                    let other = at(dir.xyz()).val0();
                                    (block_states.models[other.value].group == group, idx)
                                }
                                IfGroupOrSolid(dir, group, idx) => {
                                    let other = at(dir.xyz()).val0();
                                    (block_states.models[other.value].group == group ||
                                     block_states.get_opacity(other).is_opaque(), idx)
                                }*/
                            };
                            if cond {
                                i += 1;
                            } else {
                                i = idx as uint;
                            }
                        }
                        result
                    }
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
                            let (neighbor, _) = at(cull_face.direction());
                            if block_states.get_opacity(neighbor).is_opaque() {
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

                        let rounded_vertex_xyz = vertex.xyz.map(|x| x.round() as i32);
                        let [dx, dy, dz] = rounded_vertex_xyz;
                        for &dx in [dx - 1, dx].iter() {
                            for &dz in [dz - 1, dz].iter() {
                                for &dy in [dy - 1, dy].iter() {
                                    let (neighbor, light_level) = at([dx, dy, dz]);
                                    let light_level = max(light_level.block_light(),
                                                          light_level.sky_light());
                                    let mut light_level = light_level as f32;

                                    let use_block = match face.ao_face {
                                        Some(ao_face) => {
                                            let mut above = true;
                                            for (i, &a) in ao_face.direction().iter().enumerate() {
                                                let da = [dx, dy, dz][i];
                                                let va = rounded_vertex_xyz[i];
                                                let above_da = match a {
                                                    -1 => va - 1,
                                                    1 => va,
                                                    _ => da
                                                };
                                                if da != above_da {
                                                    above = false;
                                                    break;
                                                }
                                            }

                                            if above && block_states.get_opacity(neighbor).is_solid() {
                                                light_level = 0.0;
                                            }

                                            above
                                        }
                                        None => {
                                            !block_states.get_opacity(neighbor).is_opaque()
                                        }
                                    };

                                    if use_block {
                                        sum_light_level += light_level;
                                        num_light_level += 1.0;
                                    }
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

                        let light_factor = 0.2 + if num_light_level != 0.0 {
                            sum_light_level / num_light_level / 15.0 * 0.8
                        } else { 0.0 };

                        // Up, North and South, East and West, Down have different lighting.
                        let light_factor = light_factor * match face.ao_face {
                            Some(ao_face) => match ao_face {
                                cube::Up => 1.0,
                                cube::North | cube::South => 0.8,
                                cube::East | cube::West => 0.6,
                                cube::Down => 0.5
                            },
                            None => 1.0
                        };

                        Vertex {
                            xyz: vec3_add(block_xyz, vertex.xyz),
                            uv: vertex.uv,
                            // No clue why the difference of 2 exists.
                            rgb: rgb.map(|x| x * light_factor / num_colors - 2.0 / 255.0)
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
