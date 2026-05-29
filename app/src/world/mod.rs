use bevy::prelude::*;
use noise::{NoiseFn, Perlin};
use std::collections::HashMap;

use crate::core::{
    local_index, BlockId, ChunkPos, LocalPos, CHUNK_SIZE_I32, CHUNK_SIZE_USIZE, CHUNK_VOLUME,
};

const HEIGHT_NOISE_SCALE: f64 = 0.015;
const DETAIL_NOISE_SCALE: f64 = 0.06;
const BIOME_NOISE_SCALE: f64 = 0.01;
const TREE_NOISE_SCALE: f64 = 0.07;
const ORE_NOISE_SCALE: f64 = 0.08;
const BASE_HEIGHT: i32 = 10;
const HEIGHT_AMPLITUDE: i32 = 14;
const SEA_LEVEL: i32 = 11;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Biome {
    Plains,
    Beach,
}

struct TerrainNoise {
    biome: Perlin,
    height: Perlin,
    detail: Perlin,
    tree: Perlin,
    ore: Perlin,
}

impl TerrainNoise {
    fn new() -> Self {
        Self {
            biome: Perlin::new(2026),
            height: Perlin::new(1337),
            detail: Perlin::new(7331),
            tree: Perlin::new(4242),
            ore: Perlin::new(9001),
        }
    }
}

fn biome_at(noise: &TerrainNoise, x: i32, z: i32) -> Biome {
    let v = noise
        .biome
        .get([x as f64 * BIOME_NOISE_SCALE, z as f64 * BIOME_NOISE_SCALE]);
    if v > 0.25 {
        Biome::Beach
    } else {
        Biome::Plains
    }
}

fn terrain_height(noise: &TerrainNoise, x: i32, z: i32) -> i32 {
    let low = noise
        .height
        .get([x as f64 * HEIGHT_NOISE_SCALE, z as f64 * HEIGHT_NOISE_SCALE]);
    let detail = noise
        .detail
        .get([x as f64 * DETAIL_NOISE_SCALE, z as f64 * DETAIL_NOISE_SCALE])
        * 0.35;
    let normalized = ((low + detail + 1.0) * 0.5).clamp(0.0, 1.0);
    BASE_HEIGHT + (normalized * HEIGHT_AMPLITUDE as f64) as i32
}

fn has_tree(noise: &TerrainNoise, x: i32, z: i32, biome: Biome, ground_height: i32) -> bool {
    if biome != Biome::Plains || ground_height <= SEA_LEVEL + 1 {
        return false;
    }

    if x.rem_euclid(5) != 0 || z.rem_euclid(5) != 0 {
        return false;
    }

    noise
        .tree
        .get([x as f64 * TREE_NOISE_SCALE, z as f64 * TREE_NOISE_SCALE])
        > 0.4
}

fn has_coal_ore(noise: &TerrainNoise, x: i32, y: i32, z: i32, ground_height: i32) -> bool {
    if y >= ground_height - 3 || y > SEA_LEVEL {
        return false;
    }

    noise.ore.get([
        x as f64 * ORE_NOISE_SCALE,
        y as f64 * ORE_NOISE_SCALE,
        z as f64 * ORE_NOISE_SCALE,
    ]) > 0.56
}

fn generate_block_at(noise: &TerrainNoise, world_x: i32, world_y: i32, world_z: i32) -> BlockId {
    let biome = biome_at(noise, world_x, world_z);
    let ground_height = terrain_height(noise, world_x, world_z);
    let surface_block = match biome {
        Biome::Plains => BlockId::Grass,
        Biome::Beach => BlockId::Sand,
    };

    let mut block = if world_y > ground_height {
        if world_y <= SEA_LEVEL {
            BlockId::Water
        } else {
            BlockId::Air
        }
    } else {
        let depth = ground_height - world_y;
        if depth == 0 {
            surface_block
        } else if depth <= 3 {
            if biome == Biome::Beach {
                BlockId::Sand
            } else {
                BlockId::Dirt
            }
        } else {
            BlockId::Stone
        }
    };

    if block == BlockId::Stone && has_coal_ore(noise, world_x, world_y, world_z, ground_height) {
        block = BlockId::CoalOre;
    }

    if has_tree(noise, world_x, world_z, biome, ground_height) {
        let trunk_base = ground_height + 1;
        let trunk_top = trunk_base + 3;

        if (trunk_base..=trunk_top).contains(&world_y) {
            block = BlockId::Wood;
        } else {
            let leaf_center = IVec3::new(world_x, trunk_top, world_z);
            let pos = IVec3::new(world_x, world_y, world_z);
            let delta = pos - leaf_center;
            let horizontal = delta.x.abs() + delta.z.abs();
            let leaf_band = (trunk_top - 1)..=(trunk_top + 1);
            if leaf_band.contains(&world_y) && horizontal <= 2 && block == BlockId::Air {
                block = BlockId::Leaves;
            }
        }
    }

    block
}

#[derive(Clone)]
pub struct Chunk {
    pub blocks: Vec<BlockId>,
    pub dirty: bool,
}

#[derive(Resource)]
pub struct WorldState {
    pub chunks: HashMap<ChunkPos, Chunk>,
    pub chunk_entities: HashMap<ChunkPos, Entity>,
    pub chunk_meshes: HashMap<ChunkPos, Handle<Mesh>>,
    pub water_entities: HashMap<ChunkPos, Entity>,
    pub water_meshes: HashMap<ChunkPos, Handle<Mesh>>,
    pub material: Handle<StandardMaterial>,
    pub water_material: Handle<StandardMaterial>,
}

impl Chunk {
    pub fn generate(chunk_pos: ChunkPos) -> Self {
        let mut blocks = vec![BlockId::Air; CHUNK_VOLUME];
        let terrain_noise = TerrainNoise::new();

        for x in 0..CHUNK_SIZE_USIZE {
            for y in 0..CHUNK_SIZE_USIZE {
                for z in 0..CHUNK_SIZE_USIZE {
                    let world_x = chunk_pos.x * CHUNK_SIZE_I32 + x as i32;
                    let world_y = chunk_pos.y * CHUNK_SIZE_I32 + y as i32;
                    let world_z = chunk_pos.z * CHUNK_SIZE_I32 + z as i32;
                    let block = generate_block_at(&terrain_noise, world_x, world_y, world_z);
                    let index = local_index(LocalPos { x, y, z });
                    blocks[index] = block;
                }
            }
        }

        Self {
            blocks,
            dirty: true,
        }
    }

    pub fn get_local(&self, local: LocalPos) -> BlockId {
        self.blocks[local_index(local)]
    }

    pub fn is_solid_local(&self, local: LocalPos) -> bool {
        self.get_local(local).is_solid()
    }

    pub fn set_local(&mut self, local: LocalPos, block: BlockId) {
        let index = local_index(local);
        if self.blocks[index] != block {
            self.blocks[index] = block;
            self.dirty = true;
        }
    }
}

impl WorldState {
    pub fn new(material: Handle<StandardMaterial>) -> Self {
        Self::new_with_water_material(material.clone(), material)
    }

    pub fn new_with_water_material(
        material: Handle<StandardMaterial>,
        water_material: Handle<StandardMaterial>,
    ) -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_entities: HashMap::new(),
            chunk_meshes: HashMap::new(),
            water_entities: HashMap::new(),
            water_meshes: HashMap::new(),
            material,
            water_material,
        }
    }

    pub fn get_block_world(&self, world_pos: IVec3) -> BlockId {
        let (chunk_pos, local_pos) = crate::core::world_to_chunk_local(world_pos);
        if let Some(chunk) = self.chunks.get(&chunk_pos) {
            chunk.get_local(local_pos)
        } else {
            BlockId::Air
        }
    }

    pub fn set_block_world(&mut self, world_pos: IVec3, block: BlockId) -> bool {
        let (chunk_pos, local_pos) = crate::core::world_to_chunk_local(world_pos);
        let old = self
            .chunks
            .get(&chunk_pos)
            .map(|chunk| chunk.get_local(local_pos))
            .unwrap_or(BlockId::Air);

        if old == block {
            return false;
        }

        let Some(chunk) = self.chunks.get_mut(&chunk_pos) else {
            return false;
        };

        chunk.set_local(local_pos, block);

        self.mark_chunk_and_neighbors_dirty(chunk_pos, local_pos);
        true
    }

    fn mark_chunk_and_neighbors_dirty(&mut self, chunk_pos: ChunkPos, local: LocalPos) {
        if let Some(chunk) = self.chunks.get_mut(&chunk_pos) {
            chunk.dirty = true;
        }

        if local.x == 0 {
            self.mark_dirty(ChunkPos {
                x: chunk_pos.x - 1,
                y: chunk_pos.y,
                z: chunk_pos.z,
            });
        }
        if local.x == CHUNK_SIZE_USIZE - 1 {
            self.mark_dirty(ChunkPos {
                x: chunk_pos.x + 1,
                y: chunk_pos.y,
                z: chunk_pos.z,
            });
        }
        if local.y == 0 {
            self.mark_dirty(ChunkPos {
                x: chunk_pos.x,
                y: chunk_pos.y - 1,
                z: chunk_pos.z,
            });
        }
        if local.y == CHUNK_SIZE_USIZE - 1 {
            self.mark_dirty(ChunkPos {
                x: chunk_pos.x,
                y: chunk_pos.y + 1,
                z: chunk_pos.z,
            });
        }
        if local.z == 0 {
            self.mark_dirty(ChunkPos {
                x: chunk_pos.x,
                y: chunk_pos.y,
                z: chunk_pos.z - 1,
            });
        }
        if local.z == CHUNK_SIZE_USIZE - 1 {
            self.mark_dirty(ChunkPos {
                x: chunk_pos.x,
                y: chunk_pos.y,
                z: chunk_pos.z + 1,
            });
        }
    }

    fn mark_dirty(&mut self, pos: ChunkPos) {
        if let Some(chunk) = self.chunks.get_mut(&pos) {
            chunk.dirty = true;
        }
    }
}

#[cfg(test)]
mod test;
