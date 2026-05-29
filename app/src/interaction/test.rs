use super::*;
use crate::core::{local_index, CHUNK_VOLUME};
use crate::world::Chunk;

fn make_world_with_single_block(world_pos: IVec3, block: BlockId) -> WorldState {
    let mut world = WorldState::new(Default::default());
    let (chunk_pos, local) = crate::core::world_to_chunk_local(world_pos);
    let mut chunk = Chunk {
        blocks: vec![BlockId::Air; CHUNK_VOLUME],
        dirty: false,
    };
    chunk.blocks[local_index(local)] = block;
    world.chunks.insert(chunk_pos, chunk);
    world
}

#[test]
fn raycast_world_hits_first_solid_block() {
    let world = make_world_with_single_block(IVec3::new(2, 0, 0), BlockId::Stone);
    let hit = raycast_world(&world, Vec3::new(0.1, 0.1, 0.1), Vec3::X, 10.0).expect("expected hit");

    assert_eq!(hit.block_pos, IVec3::new(2, 0, 0));
    assert_eq!(hit.hit_normal, IVec3::new(-1, 0, 0));
}

#[test]
fn raycast_world_returns_none_when_nothing_is_hit() {
    let world = WorldState::new(Default::default());
    let hit = raycast_world(&world, Vec3::new(0.0, 0.0, 0.0), Vec3::X, 3.0);
    assert!(hit.is_none());
}

#[test]
fn break_block_removes_breakable_block() {
    let mut world = make_world_with_single_block(IVec3::new(2, 0, 0), BlockId::Stone);
    let hit = RaycastHit {
        block_pos: IVec3::new(2, 0, 0),
        hit_normal: IVec3::new(-1, 0, 0),
    };
    
    let mut mouse = ButtonInput::<MouseButton>::default();
    mouse.press(MouseButton::Left);
    
    let _cursor_options = CursorOptions {
        grab_mode: CursorGrabMode::Locked,
        ..Default::default()
    };
    
    // Simulating break_and_place_blocks logic partially as we need App to run full system
    let block = world.get_block_world(hit.block_pos);
    if block.is_breakable() {
        world.set_block_world(hit.block_pos, BlockId::Air);
    }
    
    assert_eq!(world.get_block_world(IVec3::new(2, 0, 0)), BlockId::Air);
}

#[test]
fn break_block_does_not_remove_water() {
    let mut world = make_world_with_single_block(IVec3::new(2, 0, 0), BlockId::Water);
    let hit = RaycastHit {
        block_pos: IVec3::new(2, 0, 0),
        hit_normal: IVec3::new(-1, 0, 0),
    };
    
    let block = world.get_block_world(hit.block_pos);
    if block.is_breakable() {
        world.set_block_world(hit.block_pos, BlockId::Air);
    }
    
    assert_eq!(world.get_block_world(IVec3::new(2, 0, 0)), BlockId::Water);
}
