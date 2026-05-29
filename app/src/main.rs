use bevy::prelude::*;
use std::collections::HashSet;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;
use std::thread;

mod core;
mod interaction;
mod meshing;
mod player;
mod world;

const STREAM_RADIUS_XZ: i32 = 2;
const STREAM_RADIUS_Y: i32 = 1;
use crate::core::{world_to_chunk_local, ChunkPos};
use crate::interaction::{
    break_and_place_blocks, select_hotbar_slot, update_selected_block_target, HotbarState,
    SelectedBlockTarget,
};
use crate::meshing::{build_chunk_mesh, generate_block_texture_atlas};
use crate::player::{collides_with_blocks, fps_look, fps_move, lock_cursor, EYE_HEIGHT, FpsCamera, PlayerPhysics, PLAYER_HEIGHT};
use crate::world::{Chunk, WorldState};

#[derive(Resource)]
struct ChunkGenerationQueue {
    sender: Sender<(ChunkPos, Chunk)>,
    receiver: Mutex<Receiver<(ChunkPos, Chunk)>>,
    in_flight: HashSet<ChunkPos>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "RustCraft".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                lock_cursor,
                fps_look,
                fps_move,
                select_hotbar_slot,
                update_selected_block_target,
                break_and_place_blocks,
                stream_chunks,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let spawn = find_spawn_position(8, 24);

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 10_000.0,
            ..default()
        },
        Transform::from_xyz(24.0, 32.0, 24.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Msaa::Sample4,
        Transform::from_translation(spawn).looking_at(spawn + Vec3::new(0.0, 0.0, -1.0), Vec3::Y),
        FpsCamera {
            yaw: 0.0,
            pitch: -0.3,
        },
        PlayerPhysics::default(),
    ));

    let atlas = images.add(generate_block_texture_atlas());
    let cube_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(atlas),
        alpha_mode: AlphaMode::AlphaToCoverage,
        ..default()
    });

    let (sender, receiver) = channel();
    commands.insert_resource(ChunkGenerationQueue {
        sender,
        receiver: Mutex::new(receiver),
        in_flight: HashSet::new(),
    });

    commands.insert_resource(WorldState::new(cube_material));
    commands.insert_resource(HotbarState::default());
    commands.insert_resource(SelectedBlockTarget(None));
}

fn find_spawn_position(x: i32, z: i32) -> Vec3 {
    const MIN_Y: i32 = -32;
    const MAX_Y: i32 = 96;
    const SEARCH_RADIUS: i32 = 8;

    for radius in 0..=SEARCH_RADIUS {
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                if radius > 0 && dx.abs() != radius && dz.abs() != radius {
                    continue;
                }

                if let Some(spawn) = find_spawn_in_column(x + dx, z + dz, MIN_Y, MAX_Y) {
                    return spawn;
                }
            }
        }
    }

    Vec3::new(x as f32 + 0.5, (MAX_Y + 2) as f32 + EYE_HEIGHT, z as f32 + 0.5)
}

fn find_spawn_in_column(x: i32, z: i32, min_y: i32, max_y: i32) -> Option<Vec3> {
    let mut highest_solid = None;
    for y in min_y..=max_y {
        if generated_block_at(IVec3::new(x, y, z)).is_solid() {
            highest_solid = Some(y);
        }
    }

    let top = highest_solid?;
    for stand_y in (top + 1..=max_y - 1).rev() {
        let below = generated_block_at(IVec3::new(x, stand_y - 1, z));
        let feet = generated_block_at(IVec3::new(x, stand_y, z));
        let head = generated_block_at(IVec3::new(x, stand_y + 1, z));

        if !below.is_solid() || feet != core::BlockId::Air || head != core::BlockId::Air {
            continue;
        }

        let candidate = Vec3::new(x as f32 + 0.5, stand_y as f32 + EYE_HEIGHT, z as f32 + 0.5);
        if !generated_world_collides(candidate) {
            return Some(candidate + Vec3::new(0f32, PLAYER_HEIGHT, 0f32));
        }
    }

    None
}

fn generated_world_collides(camera_position: Vec3) -> bool {
    collides_with_blocks(camera_position, false, |world_pos| {
        generated_block_at(world_pos).is_solid()
    })
}

fn generated_block_at(world_pos: IVec3) -> core::BlockId {
    let (chunk_pos, local_pos) = world_to_chunk_local(world_pos);
    let chunk = Chunk::generate(chunk_pos);
    chunk.get_local(local_pos)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn spawn_position_is_above_solid_block_with_headroom() {
        let spawn = find_spawn_position(8, 24);
        let feet_world = IVec3::new(
            spawn.x.floor() as i32,
            (spawn.y - EYE_HEIGHT).floor() as i32,
            spawn.z.floor() as i32,
        );

        let below = generated_block_at(feet_world - IVec3::Y);
        let feet = generated_block_at(feet_world);
        let head = generated_block_at(feet_world + IVec3::Y);

        assert!(below.is_solid(), "spawn must stand on a solid block");
        assert_eq!(feet, core::BlockId::Air, "spawn feet space must be air");
        assert_eq!(head, core::BlockId::Air, "spawn head space must be air");
    }

    #[test]
    fn spawn_position_does_not_collide_with_generated_world() {
        let spawn = find_spawn_position(8, 24);
        assert!(!generated_world_collides(spawn));
    }
}

fn stream_chunks(
    mut commands: Commands,
    mut world: ResMut<WorldState>,
    mut generation_queue: ResMut<ChunkGenerationQueue>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_query: Query<&Transform, With<FpsCamera>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    let player_world = camera_transform.translation.floor().as_ivec3();
    let (center_chunk, _) = world_to_chunk_local(player_world);

    let mut desired = HashSet::new();
    for dx in -STREAM_RADIUS_XZ..=STREAM_RADIUS_XZ {
        for dy in -STREAM_RADIUS_Y..=STREAM_RADIUS_Y {
            for dz in -STREAM_RADIUS_XZ..=STREAM_RADIUS_XZ {
                desired.insert(ChunkPos {
                    x: center_chunk.x + dx,
                    y: center_chunk.y + dy,
                    z: center_chunk.z + dz,
                });
            }
        }
    }

    let generated_chunks: Vec<(ChunkPos, Chunk)> = if let Ok(receiver) = generation_queue.receiver.lock() {
        receiver.try_iter().collect()
    } else {
        Vec::new()
    };

    for (chunk_pos, chunk) in generated_chunks {
        generation_queue.in_flight.remove(&chunk_pos);
        if desired.contains(&chunk_pos) {
            world.chunks.entry(chunk_pos).or_insert(chunk);
        }
    }

    for chunk_pos in &desired {
        if !world.chunks.contains_key(chunk_pos) && !generation_queue.in_flight.contains(chunk_pos) {
            generation_queue.in_flight.insert(*chunk_pos);
            let sender = generation_queue.sender.clone();
            let chunk_pos_copy = *chunk_pos;
            thread::spawn(move || {
                let generated = Chunk::generate(chunk_pos_copy);
                let _ = sender.send((chunk_pos_copy, generated));
            });
        }
    }

    let to_unload: Vec<ChunkPos> = world
        .chunks
        .keys()
        .filter(|pos| !desired.contains(*pos))
        .copied()
        .collect();

    for chunk_pos in to_unload {
        generation_queue.in_flight.remove(&chunk_pos);
        if let Some(entity) = world.chunk_entities.remove(&chunk_pos) {
            commands.entity(entity).despawn();
        }
        world.chunk_meshes.remove(&chunk_pos);
        world.chunks.remove(&chunk_pos);
    }

    let dirty_chunks: Vec<ChunkPos> = world
        .chunks
        .iter()
        .filter_map(|(pos, chunk)| if chunk.dirty { Some(*pos) } else { None })
        .collect();

    for chunk_pos in dirty_chunks {
        let Some(mesh) = build_chunk_mesh(chunk_pos, &world.chunks) else {
            if let Some(entity) = world.chunk_entities.remove(&chunk_pos) {
                commands.entity(entity).despawn();
            }
            world.chunk_meshes.remove(&chunk_pos);
            if let Some(chunk) = world.chunks.get_mut(&chunk_pos) {
                chunk.dirty = false;
            }
            continue;
        };

        if let Some(handle) = world.chunk_meshes.get(&chunk_pos).cloned() {
            if let Some(existing_mesh) = meshes.get_mut(&handle) {
                *existing_mesh = mesh;
            } else {
                let new_handle = meshes.add(mesh);
                world.chunk_meshes.insert(chunk_pos, new_handle.clone());
                if let Some(entity) = world.chunk_entities.get(&chunk_pos).copied() {
                    commands.entity(entity).insert(Mesh3d(new_handle));
                }
            }
        } else {
            let mesh_handle = meshes.add(mesh);
            let entity = commands
                .spawn((
                    Mesh3d(mesh_handle.clone()),
                    MeshMaterial3d(world.material.clone()),
                    Transform::default(),
                ))
                .id();
            world.chunk_entities.insert(chunk_pos, entity);
            world.chunk_meshes.insert(chunk_pos, mesh_handle);
        }

        if let Some(chunk) = world.chunks.get_mut(&chunk_pos) {
            chunk.dirty = false;
        }
    }
}
