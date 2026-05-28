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
fn hotbar_default_selected_block_matches_first_slot() {
    let hotbar = HotbarState::default();
    assert_eq!(hotbar.selected, 0);
    assert_eq!(hotbar.selected_block(), hotbar.slots[0]);
}
