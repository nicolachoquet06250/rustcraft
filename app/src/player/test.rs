use super::*;
use crate::core::{local_index, BlockId, ChunkPos, LocalPos, CHUNK_VOLUME};
use crate::world::Chunk;
use crate::world::WorldState;
use std::time::Duration;

#[test]
fn lock_cursor_locks_on_left_click() {
    let mut app = App::new();
    app.insert_resource(ButtonInput::<MouseButton>::default());
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.add_systems(Update, lock_cursor);

    app.world_mut().spawn((CursorOptions::default(), PrimaryWindow));
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);

    app.update();

    let cursor = {
        let world = app.world_mut();
        world
            .query::<&CursorOptions>()
            .single(world)
            .expect("primary window cursor options should exist")
            .clone()
    };
    assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
    assert!(!cursor.visible);
}

#[test]
fn lock_cursor_unlocks_on_escape() {
    let mut app = App::new();
    app.insert_resource(ButtonInput::<MouseButton>::default());
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.add_systems(Update, lock_cursor);

    app.world_mut().spawn((
        CursorOptions {
            grab_mode: CursorGrabMode::Locked,
            visible: false,
            ..default()
        },
        PrimaryWindow,
    ));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::Escape);

    app.update();

    let cursor = {
        let world = app.world_mut();
        world
            .query::<&CursorOptions>()
            .single(world)
            .expect("primary window cursor options should exist")
            .clone()
    };
    assert_eq!(cursor.grab_mode, CursorGrabMode::None);
    assert!(cursor.visible);
}

#[test]
fn fps_move_advances_camera_with_forward_input() {
    let mut app = App::new();
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(Time::<()>::default());
    app.insert_resource(WorldState::new(Default::default()));
    app.add_systems(Update, fps_move);

    let entity = app
        .world_mut()
        .spawn((
            Transform::default(),
            FpsCamera {
                yaw: 0.0,
                pitch: 0.0,
            },
            PlayerPhysics::default(),
        ))
        .id();

    app.world_mut()
        .resource_mut::<Time<()>>()
        .advance_by(Duration::from_secs_f32(1.0));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);

    app.update();

    let transform = app.world().entity(entity).get::<Transform>().unwrap();
    assert!(transform.translation.z < -9.0);
}

#[test]
fn collides_with_world_ignores_water_blocks() {
    let mut chunk = Chunk {
        blocks: vec![BlockId::Air; CHUNK_VOLUME],
        dirty: false,
    };
    let water_local = LocalPos { x: 0, y: 0, z: 0 };
    chunk.blocks[local_index(water_local)] = BlockId::Water;

    let mut chunks = std::collections::HashMap::new();
    chunks.insert(ChunkPos { x: 0, y: 0, z: 0 }, chunk);
    let world = WorldState {
        chunks,
        chunk_entities: std::collections::HashMap::new(),
        chunk_meshes: std::collections::HashMap::new(),
        material: default(),
    };

    let camera_inside_water = Vec3::new(0.5, EYE_HEIGHT, 0.5);
    assert!(!collides_with_world(&world, camera_inside_water, false));
}

#[test]
fn sneak_reduces_speed() {
    let mut app = App::new();
    app.insert_resource(ButtonInput::<KeyCode>::default());
    app.insert_resource(Time::<()>::default());
    app.insert_resource(WorldState::new(Default::default()));
    app.add_systems(Update, fps_move);

    let entity = app
        .world_mut()
        .spawn((
            Transform::default(),
            FpsCamera {
                yaw: 0.0,
                pitch: 0.0,
            },
            PlayerPhysics::default(),
        ))
        .id();

    app.world_mut()
        .resource_mut::<Time<()>>()
        .advance_by(Duration::from_secs_f32(1.0));
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::KeyW);
    app.world_mut()
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ShiftLeft);

    app.update();

    let transform = app.world().entity(entity).get::<Transform>().unwrap();
    // Vitesse normale = 10, SNEAK_SPEED = 3.
    // Après 1s, z devrait être environ -3.0
    assert!(transform.translation.z < -2.9 && transform.translation.z > -3.1);
}