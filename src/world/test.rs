use super::*;

#[test]
fn chunk_generate_sets_dirty_and_expected_volume() {
    let chunk = Chunk::generate(ChunkPos { x: 0, y: 0, z: 0 });

    assert!(chunk.dirty);
    assert_eq!(chunk.blocks.len(), CHUNK_VOLUME);
}

#[test]
fn chunk_generate_contains_surface_and_subsurface_materials() {
    let chunk = Chunk::generate(ChunkPos { x: 0, y: 0, z: 0 });

    assert!(chunk.blocks.iter().any(|b| *b == BlockId::Grass || *b == BlockId::Sand));
    assert!(chunk.blocks.iter().any(|b| *b == BlockId::Dirt));
    assert!(chunk.blocks.iter().any(|b| *b == BlockId::Stone || *b == BlockId::CoalOre));
    assert!(chunk.blocks.iter().any(|b| *b == BlockId::Water || *b == BlockId::Air));
}

#[test]
fn chunk_generate_with_high_chunk_y_is_only_empty_or_tree_part() {
    let chunk = Chunk::generate(ChunkPos { x: 0, y: 1, z: 0 });
    assert!(!chunk.blocks.iter().any(|block| *block == BlockId::Water));
    assert!(!chunk.blocks.iter().any(|block| *block == BlockId::CoalOre));
}

#[test]
fn is_solid_local_matches_get_local_result() {
    let chunk = Chunk::generate(ChunkPos { x: 0, y: 0, z: 0 });
    for y in 0..CHUNK_SIZE_USIZE {
        let local = LocalPos { x: 0, y, z: 0 };
        assert_eq!(chunk.is_solid_local(local), chunk.get_local(local).is_solid());
    }
}

#[test]
fn generate_block_at_has_water_band_and_tree_or_ore_candidates() {
    let mut saw_water = false;
    let mut saw_tree_part = false;
    let mut saw_ore = false;
    let noise = TerrainNoise::new();

    for x in -48..=48 {
        for z in -48..=48 {
            let height = terrain_height(&noise, x, z);
            if height < SEA_LEVEL {
                let block = generate_block_at(&noise, x, SEA_LEVEL, z);
                if block == BlockId::Water {
                    saw_water = true;
                }
            }

            for y in -4..=24 {
                match generate_block_at(&noise, x, y, z) {
                    BlockId::Wood | BlockId::Leaves => saw_tree_part = true,
                    BlockId::CoalOre => saw_ore = true,
                    _ => {}
                }
            }
        }
    }

    assert!(saw_water, "expected at least one water cell at sea level");
    assert!(saw_tree_part, "expected at least one generated tree block");
    assert!(saw_ore, "expected at least one generated ore block");
}

#[test]
fn set_block_world_updates_block_and_marks_neighbors_dirty_on_chunk_border() {
    let mut world = WorldState::new(Default::default());

    let center = ChunkPos { x: 0, y: 0, z: 0 };
    let left = ChunkPos { x: -1, y: 0, z: 0 };
    world.chunks.insert(
        center,
        Chunk {
            blocks: vec![BlockId::Stone; CHUNK_VOLUME],
            dirty: false,
        },
    );
    world.chunks.insert(
        left,
        Chunk {
            blocks: vec![BlockId::Stone; CHUNK_VOLUME],
            dirty: false,
        },
    );

    let changed = world.set_block_world(IVec3::new(0, 1, 1), BlockId::Air);
    assert!(changed);
    assert_eq!(world.get_block_world(IVec3::new(0, 1, 1)), BlockId::Air);
    assert!(world.chunks.get(&center).unwrap().dirty);
    assert!(world.chunks.get(&left).unwrap().dirty);
}