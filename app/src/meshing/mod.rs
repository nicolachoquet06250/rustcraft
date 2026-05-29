use crate::core::{BlockId, ChunkPos, LocalPos, CHUNK_SIZE_I32, CHUNK_SIZE_USIZE};
use crate::world::Chunk;
use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, PrimitiveTopology, TextureDimension, TextureFormat,
};
use noise::{NoiseFn, Perlin};
use std::collections::HashMap;

pub const ATLAS_TILE_SIZE: u32 = 16;
pub const ATLAS_GRID_SIZE: u32 = 4;
pub const ATLAS_SIZE: u32 = ATLAS_TILE_SIZE * ATLAS_GRID_SIZE;

const TILE_GRASS_TOP: u32 = 0;
const TILE_GRASS_SIDE: u32 = 1;
const TILE_DIRT: u32 = 2;
const TILE_STONE: u32 = 3;
const TILE_SAND: u32 = 4;
const TILE_WATER: u32 = 5;
const TILE_WOOD: u32 = 6;
const TILE_LEAVES: u32 = 7;
const TILE_COAL_ORE: u32 = 8;
const WATER_ALPHA: u8 = 204;
const WATER_SURFACE_Y_OFFSET: f32 = -0.08;
const WATER_OPACITY: f32 = 0.58;

fn block_base_color(block: BlockId) -> [u8; 3] {
    match block {
        BlockId::Air => [0, 0, 0],
        BlockId::Grass => [84, 184, 71],
        BlockId::Dirt => [125, 79, 43],
        BlockId::Stone => [140, 145, 153],
        BlockId::Sand => [222, 209, 140],
        BlockId::Water => [56, 117, 235],
        BlockId::Wood => [114, 71, 35],
        BlockId::Leaves => [56, 143, 46],
        BlockId::CoalOre => [53, 53, 58],
    }
}

fn block_face_tile_index(block: BlockId, normal: IVec3) -> Option<u32> {
    match block {
        BlockId::Air => None,
        BlockId::Grass => {
            if normal.y > 0 {
                Some(TILE_GRASS_TOP)
            } else if normal.y < 0 {
                Some(TILE_DIRT)
            } else {
                Some(TILE_GRASS_SIDE)
            }
        }
        BlockId::Dirt => Some(TILE_DIRT),
        BlockId::Stone => Some(TILE_STONE),
        BlockId::Sand => Some(TILE_SAND),
        BlockId::Water => Some(TILE_WATER),
        BlockId::Wood => Some(TILE_WOOD),
        BlockId::Leaves => Some(TILE_LEAVES),
        BlockId::CoalOre => Some(TILE_COAL_ORE),
    }
}

fn tile_uv_rect(tile: u32) -> [[f32; 2]; 4] {
    let tile_x = tile % ATLAS_GRID_SIZE;
    let tile_y = tile / ATLAS_GRID_SIZE;
    let atlas_size = ATLAS_SIZE as f32;
    let tile_size = ATLAS_TILE_SIZE as f32;
    let pad = 0.001;

    let u0 = (tile_x as f32 * tile_size + pad) / atlas_size;
    let v0 = (tile_y as f32 * tile_size + pad) / atlas_size;
    let u1 = ((tile_x + 1) as f32 * tile_size - pad) / atlas_size;
    let v1 = ((tile_y + 1) as f32 * tile_size - pad) / atlas_size;

    [[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
}

pub fn create_terrain_material(atlas: Handle<Image>) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(atlas),
        perceptual_roughness: 0.9,
        ..default()
    }
}

pub fn create_water_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgba(0.16, 0.42, 0.95, WATER_OPACITY),
        alpha_mode: AlphaMode::AlphaToCoverage,
        cull_mode: None,
        double_sided: true,
        perceptual_roughness: 0.18,
        reflectance: 0.55,
        ..default()
    }
}

pub fn generate_block_texture_atlas() -> Image {
    let mut data = vec![0u8; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize];
    let tiles = [
        TILE_GRASS_TOP,
        TILE_GRASS_SIDE,
        TILE_DIRT,
        TILE_STONE,
        TILE_SAND,
        TILE_WATER,
        TILE_WOOD,
        TILE_LEAVES,
        TILE_COAL_ORE,
    ];

    for tile in tiles {
        let tile_x = tile % ATLAS_GRID_SIZE;
        let tile_y = tile / ATLAS_GRID_SIZE;
        let base = match tile {
            TILE_GRASS_TOP => block_base_color(BlockId::Grass),
            TILE_GRASS_SIDE => block_base_color(BlockId::Dirt),
            TILE_DIRT => block_base_color(BlockId::Dirt),
            TILE_STONE => block_base_color(BlockId::Stone),
            TILE_SAND => block_base_color(BlockId::Sand),
            TILE_WATER => block_base_color(BlockId::Water),
            TILE_WOOD => block_base_color(BlockId::Wood),
            TILE_LEAVES => block_base_color(BlockId::Leaves),
            TILE_COAL_ORE => block_base_color(BlockId::CoalOre),
            _ => [255, 0, 255],
        };
        let noise = Perlin::new(2000 + tile);
        for py in 0..ATLAS_TILE_SIZE {
            for px in 0..ATLAS_TILE_SIZE {
                let x = tile_x * ATLAS_TILE_SIZE + px;
                let y = tile_y * ATLAS_TILE_SIZE + py;
                let i = ((y * ATLAS_SIZE + x) * 4) as usize;

                let n = noise.get([px as f64 * 0.25, py as f64 * 0.25]) as f32;
                let shade = match tile {
                    TILE_WATER => 0.92 + n * 0.08,
                    TILE_GRASS_TOP => {
                        let stepped = (((n + 1.0) * 0.5 * 5.0).floor()) / 5.0;
                        0.78 + stepped * 0.38
                    }
                    TILE_GRASS_SIDE => {
                        let stepped = (((n + 1.0) * 0.5 * 4.0).floor()) / 4.0;
                        if py < 4 {
                            0.85 + stepped * 0.20
                        } else {
                            0.72 + stepped * 0.25
                        }
                    }
                    _ => {
                        let stepped = (((n + 1.0) * 0.5 * 4.0).floor()) / 4.0;
                        0.78 + stepped * 0.44
                    }
                };

                let mut r = (base[0] as f32 * shade).clamp(0.0, 255.0);
                let mut g = (base[1] as f32 * shade).clamp(0.0, 255.0);
                let mut b = (base[2] as f32 * shade).clamp(0.0, 255.0);

                if tile == TILE_GRASS_SIDE && py < 4 {
                    r = (r * 0.55 + 84.0).clamp(0.0, 255.0);
                    g = (g * 0.45 + 170.0).clamp(0.0, 255.0);
                    b = (b * 0.45 + 58.0).clamp(0.0, 255.0);
                }

                data[i] = r as u8;
                data[i + 1] = g as u8;
                data[i + 2] = b as u8;
                data[i + 3] = if tile == TILE_WATER { WATER_ALPHA } else { 255 };
            }
        }
    }

    let mut image = Image::new_fill(
        Extent3d {
            width: ATLAS_SIZE,
            height: ATLAS_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    image
}

fn get_block_world(chunks: &HashMap<ChunkPos, Chunk>, world_pos: IVec3) -> BlockId {
    let (chunk_pos, local_pos) = crate::core::world_to_chunk_local(world_pos);
    if let Some(chunk) = chunks.get(&chunk_pos) {
        chunk.get_local(local_pos)
    } else {
        BlockId::Air
    }
}

fn should_render_face(current: BlockId, neighbor: BlockId, normal: IVec3) -> bool {
    if current == BlockId::Water {
        return normal.y > 0 && neighbor == BlockId::Air;
    }

    if neighbor == current {
        return false;
    }

    if neighbor == BlockId::Water {
        return true;
    }

    !(current.is_opaque() && neighbor.is_opaque())
}

pub fn build_chunk_terrain_mesh(
    chunk_pos: ChunkPos,
    chunks: &HashMap<ChunkPos, Chunk>,
) -> Option<Mesh> {
    let chunk = chunks.get(&chunk_pos)?;
    let chunk_origin = IVec3::new(
        chunk_pos.x * CHUNK_SIZE_I32,
        chunk_pos.y * CHUNK_SIZE_I32,
        chunk_pos.z * CHUNK_SIZE_I32,
    );

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let faces: [(IVec3, [[f32; 3]; 4]); 6] = [
        (
            IVec3::new(1, 0, 0),
            [
                [1.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
            ],
        ),
        (
            IVec3::new(-1, 0, 0),
            [
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
            ],
        ),
        (
            IVec3::new(0, 1, 0),
            [
                [0.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
        ),
        (
            IVec3::new(0, -1, 0),
            [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
            ],
        ),
        (
            IVec3::new(0, 0, 1),
            [
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
        ),
        (
            IVec3::new(0, 0, -1),
            [
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
        ),
    ];

    for lx in 0..CHUNK_SIZE_USIZE {
        for ly in 0..CHUNK_SIZE_USIZE {
            for lz in 0..CHUNK_SIZE_USIZE {
                let local = LocalPos {
                    x: lx,
                    y: ly,
                    z: lz,
                };

                let current = chunk.get_local(local);
                if matches!(current, BlockId::Air | BlockId::Water) {
                    continue;
                }

                let voxel_world = chunk_origin + IVec3::new(lx as i32, ly as i32, lz as i32);

                for (normal, corners) in faces {
                    let neighbor = voxel_world + normal;
                    let neighbor_block = get_block_world(chunks, neighbor);
                    if !should_render_face(current, neighbor_block, normal) {
                        continue;
                    }

                    let base = positions.len() as u32;
                    let tile = block_face_tile_index(current, normal).unwrap_or(TILE_DIRT);
                    let face_uvs = tile_uv_rect(tile);
                    let mut uv_index = 0;
                    for corner in corners {
                        positions.push([
                            voxel_world.x as f32 + corner[0],
                            voxel_world.y as f32 + corner[1],
                            voxel_world.z as f32 + corner[2],
                        ]);
                        normals.push([normal.x as f32, normal.y as f32, normal.z as f32]);
                        uvs.push(face_uvs[uv_index]);
                        uv_index += 1;
                    }

                    indices.extend_from_slice(&[
                        base,
                        base + 1,
                        base + 2,
                        base,
                        base + 2,
                        base + 3,
                    ]);
                }
            }
        }
    }

    if indices.is_empty() {
        return None;
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    Some(mesh)
}

pub fn build_chunk_water_mesh(
    chunk_pos: ChunkPos,
    chunks: &HashMap<ChunkPos, Chunk>,
) -> Option<Mesh> {
    let chunk = chunks.get(&chunk_pos)?;
    let chunk_origin = IVec3::new(
        chunk_pos.x * CHUNK_SIZE_I32,
        chunk_pos.y * CHUNK_SIZE_I32,
        chunk_pos.z * CHUNK_SIZE_I32,
    );

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let face_uvs = tile_uv_rect(TILE_WATER);

    for lx in 0..CHUNK_SIZE_USIZE {
        for ly in 0..CHUNK_SIZE_USIZE {
            for lz in 0..CHUNK_SIZE_USIZE {
                let local = LocalPos {
                    x: lx,
                    y: ly,
                    z: lz,
                };

                if chunk.get_local(local) != BlockId::Water {
                    continue;
                }

                let voxel_world = chunk_origin + IVec3::new(lx as i32, ly as i32, lz as i32);
                if get_block_world(chunks, voxel_world + IVec3::Y) != BlockId::Air {
                    continue;
                }

                let base = positions.len() as u32;
                let y = voxel_world.y as f32 + 1.0 + WATER_SURFACE_Y_OFFSET;
                positions.extend_from_slice(&[
                    [voxel_world.x as f32, y, voxel_world.z as f32 + 1.0],
                    [voxel_world.x as f32 + 1.0, y, voxel_world.z as f32 + 1.0],
                    [voxel_world.x as f32 + 1.0, y, voxel_world.z as f32],
                    [voxel_world.x as f32, y, voxel_world.z as f32],
                ]);
                normals.extend_from_slice(&[[0.0, 1.0, 0.0]; 4]);
                uvs.extend_from_slice(&face_uvs);
                indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
            }
        }
    }

    if indices.is_empty() {
        return None;
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    Some(mesh)
}

pub fn build_chunk_mesh(chunk_pos: ChunkPos, chunks: &HashMap<ChunkPos, Chunk>) -> Option<Mesh> {
    let terrain_mesh = build_chunk_terrain_mesh(chunk_pos, chunks);
    terrain_mesh.or_else(|| build_chunk_water_mesh(chunk_pos, chunks))
}

#[cfg(test)]
mod test;
