use bevy::prelude::IVec3;

pub const CHUNK_SIZE_I32: i32 = 16;
pub const CHUNK_SIZE_USIZE: usize = CHUNK_SIZE_I32 as usize;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockId {
    Air,
    Grass,
    Dirt,
    Stone,
    Sand,
    Water,
    Wood,
    Leaves,
    CoalOre,
}

impl BlockId {
    pub fn is_solid(self) -> bool {
        !matches!(self, Self::Air | Self::Water | Self::Leaves)
    }

    pub fn is_opaque(self) -> bool {
        !matches!(self, Self::Air | Self::Water)
    }

    pub fn is_breakable(self) -> bool {
        !matches!(self, Self::Water)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct LocalPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

pub fn local_index(local: LocalPos) -> usize {
    local.x + local.y * CHUNK_SIZE_USIZE + local.z * CHUNK_SIZE_USIZE * CHUNK_SIZE_USIZE
}

pub fn world_to_chunk_local(world: IVec3) -> (ChunkPos, LocalPos) {
    let chunk = ChunkPos {
        x: world.x.div_euclid(CHUNK_SIZE_I32),
        y: world.y.div_euclid(CHUNK_SIZE_I32),
        z: world.z.div_euclid(CHUNK_SIZE_I32),
    };

    let local = LocalPos {
        x: world.x.rem_euclid(CHUNK_SIZE_I32) as usize,
        y: world.y.rem_euclid(CHUNK_SIZE_I32) as usize,
        z: world.z.rem_euclid(CHUNK_SIZE_I32) as usize,
    };

    (chunk, local)
}

#[cfg(test)]
mod test;
