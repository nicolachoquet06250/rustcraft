use super::*;

fn make_air_chunk() -> Chunk {
    Chunk {
        blocks: vec![BlockId::Air; crate::core::CHUNK_VOLUME],
        dirty: false,
    }
}

fn make_single_block_chunk(local: LocalPos) -> Chunk {
    let mut chunk = make_air_chunk();
    let index = crate::core::local_index(local);
    chunk.blocks[index] = BlockId::Stone;
    chunk
}

fn make_two_block_chunk(a: (LocalPos, BlockId), b: (LocalPos, BlockId)) -> Chunk {
    let mut chunk = make_air_chunk();
    chunk.blocks[crate::core::local_index(a.0)] = a.1;
    chunk.blocks[crate::core::local_index(b.0)] = b.1;
    chunk
}

#[test]
fn build_chunk_mesh_returns_none_for_missing_chunk() {
    let chunks = HashMap::new();

    let mesh = build_chunk_mesh(ChunkPos { x: 0, y: 0, z: 0 }, &chunks);
    assert!(mesh.is_none());
}

#[test]
fn build_chunk_mesh_returns_none_for_all_air_chunk() {
    let mut chunks = HashMap::new();
    let pos = ChunkPos { x: 0, y: 0, z: 0 };
    chunks.insert(pos, make_air_chunk());

    let mesh = build_chunk_mesh(pos, &chunks);
    assert!(mesh.is_none());
}

#[test]
fn build_chunk_mesh_single_voxel_has_six_faces() {
    let mut chunks = HashMap::new();
    let pos = ChunkPos { x: 0, y: 0, z: 0 };
    chunks.insert(pos, make_single_block_chunk(LocalPos { x: 1, y: 1, z: 1 }));

    let mesh = build_chunk_mesh(pos, &chunks).expect("mesh should exist");
    assert_eq!(mesh.count_vertices(), 24);
}

#[test]
fn build_chunk_mesh_hides_internal_face_with_neighbor_chunk() {
    let mut chunks = HashMap::new();
    let left = ChunkPos { x: 0, y: 0, z: 0 };
    let right = ChunkPos { x: 1, y: 0, z: 0 };

    chunks.insert(
        left,
        make_single_block_chunk(LocalPos {
            x: CHUNK_SIZE_USIZE - 1,
            y: 0,
            z: 0,
        }),
    );
    chunks.insert(right, make_single_block_chunk(LocalPos { x: 0, y: 0, z: 0 }));

    let left_mesh = build_chunk_mesh(left, &chunks).expect("left mesh should exist");
    assert_eq!(left_mesh.count_vertices(), 20);
}

#[test]
fn build_chunk_mesh_keeps_solid_face_visible_when_touching_water() {
    let mut chunks = HashMap::new();
    let pos = ChunkPos { x: 0, y: 0, z: 0 };
    chunks.insert(
        pos,
        make_two_block_chunk(
            (LocalPos { x: 1, y: 1, z: 1 }, BlockId::Stone),
            (LocalPos { x: 2, y: 1, z: 1 }, BlockId::Water),
        ),
    );

    let mesh = build_chunk_mesh(pos, &chunks).expect("mesh should exist");
    assert_eq!(mesh.count_vertices(), 28);
}

#[test]
fn build_chunk_mesh_renders_only_top_face_for_water_block() {
    let mut chunks = HashMap::new();
    let pos = ChunkPos { x: 0, y: 0, z: 0 };
    let mut chunk = make_air_chunk();
    chunk.blocks[crate::core::local_index(LocalPos { x: 1, y: 1, z: 1 })] = BlockId::Water;
    chunks.insert(
        pos,
        chunk,
    );

    let mesh = build_chunk_mesh(pos, &chunks).expect("mesh should exist");
    // Seule la face supérieure (4 sommets) doit être rendue
    assert_eq!(mesh.count_vertices(), 4);
}

#[test]
fn build_chunk_mesh_uses_different_uv_tiles_for_different_blocks() {
    let mut chunks = HashMap::new();
    let pos = ChunkPos { x: 0, y: 0, z: 0 };
    chunks.insert(
        pos,
        make_two_block_chunk(
            (LocalPos { x: 1, y: 1, z: 1 }, BlockId::Grass),
            (LocalPos { x: 3, y: 1, z: 1 }, BlockId::Sand),
        ),
    );

    let mesh = build_chunk_mesh(pos, &chunks).expect("mesh should exist");
    let uvs = mesh
        .attribute(Mesh::ATTRIBUTE_UV_0)
        .expect("mesh should contain uvs");

    match uvs {
        bevy::mesh::VertexAttributeValues::Float32x2(values) => {
            let has_grass_row = values.iter().any(|uv| uv[1] <= 0.25);
            let has_sand_row = values.iter().any(|uv| uv[1] > 0.25 && uv[1] <= 0.50);
            assert!(has_grass_row, "expected UVs in grass tile row");
            assert!(has_sand_row, "expected UVs in sand tile row");
        }
        _ => panic!("unexpected uv format"),
    }
}

#[test]
fn grass_block_uses_distinct_tiles_for_top_side_and_bottom_faces() {
    let mut chunks = HashMap::new();
    let pos = ChunkPos { x: 0, y: 0, z: 0 };

    chunks.insert(
        pos,
        make_two_block_chunk(
            (LocalPos { x: 2, y: 2, z: 2 }, BlockId::Grass),
            (LocalPos { x: 6, y: 2, z: 2 }, BlockId::Stone),
        ),
    );

    let mesh = build_chunk_mesh(pos, &chunks).expect("mesh should exist");
    let uvs = mesh
        .attribute(Mesh::ATTRIBUTE_UV_0)
        .expect("mesh should contain uvs");

    match uvs {
        bevy::mesh::VertexAttributeValues::Float32x2(values) => {
            let has_top_grass = values.iter().any(|uv| uv[0] <= 0.25 && uv[1] <= 0.25);
            let has_side_grass = values.iter().any(|uv| uv[0] > 0.25 && uv[0] <= 0.50 && uv[1] <= 0.25);
            let has_bottom_dirt = values.iter().any(|uv| uv[0] > 0.50 && uv[0] <= 0.75 && uv[1] <= 0.25);

            assert!(has_top_grass, "expected top grass tile UVs");
            assert!(has_side_grass, "expected side grass tile UVs");
            assert!(has_bottom_dirt, "expected bottom dirt tile UVs");
        }
        _ => panic!("unexpected uv format"),
    }
}

#[test]
fn generated_texture_atlas_is_not_uniform() {
    let image = generate_block_texture_atlas();
    let data = image.data.expect("atlas should contain pixel data");
    let mut min = 255u8;
    let mut max = 0u8;

    for pixel in data.chunks_exact(4) {
        min = min.min(pixel[0]);
        max = max.max(pixel[0]);
    }

    assert!(max > min, "expected procedural texture atlas variation");
}

#[test]
fn generated_texture_atlas_makes_only_water_top_transparent() {
    let image = generate_block_texture_atlas();
    let data = image.data.as_ref().expect("atlas should contain pixel data");

    for tile in [
        TILE_GRASS_TOP,
        TILE_GRASS_SIDE,
        TILE_DIRT,
        TILE_STONE,
        TILE_SAND,
        TILE_WATER_TOP,
        TILE_WATER_SIDE,
        TILE_WOOD,
        TILE_LEAVES,
        TILE_COAL_ORE,
    ] {
        let tile_x = tile % ATLAS_GRID_SIZE;
        let tile_y = tile / ATLAS_GRID_SIZE;
        let x = tile_x * ATLAS_TILE_SIZE;
        let y = tile_y * ATLAS_TILE_SIZE;
        let i = ((y * ATLAS_SIZE + x) * 4 + 3) as usize;
        let expected_alpha = if tile == TILE_WATER_TOP {
            WATER_ALPHA
        } else {
            255
        };

        assert_eq!(data[i], expected_alpha, "tile {} has wrong alpha", tile);
    }
}

#[test]
fn build_chunk_mesh_triangle_winding_matches_declared_normals() {
    let mut chunks = HashMap::new();
    let pos = ChunkPos { x: 0, y: 0, z: 0 };
    chunks.insert(pos, make_single_block_chunk(LocalPos { x: 1, y: 1, z: 1 }));

    let mesh = build_chunk_mesh(pos, &chunks).expect("mesh should exist");

    let positions = match mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .expect("mesh should contain positions")
    {
        bevy::mesh::VertexAttributeValues::Float32x3(values) => values,
        _ => panic!("unexpected position format"),
    };

    let normals = match mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .expect("mesh should contain normals")
    {
        bevy::mesh::VertexAttributeValues::Float32x3(values) => values,
        _ => panic!("unexpected normal format"),
    };

    let indices = match mesh.indices().expect("mesh should contain indices") {
        bevy::mesh::Indices::U32(values) => values,
        bevy::mesh::Indices::U16(_) => panic!("unexpected u16 indices"),
    };

    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let p0 = Vec3::from_array(positions[i0]);
        let p1 = Vec3::from_array(positions[i1]);
        let p2 = Vec3::from_array(positions[i2]);
        let n = Vec3::from_array(normals[i0]);

        let triangle_normal = (p1 - p0).cross(p2 - p0);
        assert!(
            triangle_normal.dot(n) > 0.0,
            "triangle winding is opposite to declared normal"
        );
    }
}
