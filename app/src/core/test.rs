use super::*;

#[test]
fn local_index_maps_3d_to_linear_index() {
    let local = LocalPos { x: 3, y: 2, z: 1 };
    assert_eq!(local_index(local), 291);
}

#[test]
fn world_to_chunk_local_handles_positive_and_negative_coords() {
    let (chunk, local) = world_to_chunk_local(IVec3::new(-1, 16, -17));

    assert_eq!(chunk, ChunkPos { x: -1, y: 1, z: -2 });
    assert_eq!(local.x, 15);
    assert_eq!(local.y, 0);
    assert_eq!(local.z, 15);
}

#[test]
fn world_to_chunk_local_chunk_boundary_is_correct() {
    let (chunk, local) = world_to_chunk_local(IVec3::new(16, 31, 32));

    assert_eq!(chunk, ChunkPos { x: 1, y: 1, z: 2 });
    assert_eq!(local.x, 0);
    assert_eq!(local.y, 15);
    assert_eq!(local.z, 0);
}

#[test]
fn block_id_solid_and_opaque_flags_are_consistent() {
    assert!(!BlockId::Air.is_solid());
    assert!(!BlockId::Air.is_opaque());
    assert!(!BlockId::Water.is_solid());
    assert!(!BlockId::Water.is_opaque());
    assert!(!BlockId::Leaves.is_solid());
    assert!(BlockId::Leaves.is_opaque());
    assert!(BlockId::Stone.is_solid());
    assert!(BlockId::Stone.is_opaque());
}