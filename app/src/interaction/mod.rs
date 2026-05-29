use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::core::BlockId;
use crate::player::FpsCamera;
use crate::world::WorldState;

const RAYCAST_MAX_DISTANCE: f32 = 6.0;
const RAYCAST_STEP: f32 = 0.05;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RaycastHit {
    pub block_pos: IVec3,
    pub hit_normal: IVec3,
}

#[derive(Resource)]
pub struct SelectedBlockTarget(pub Option<RaycastHit>);

#[derive(Resource)]
pub struct HotbarState {
    pub slots: [BlockId; 9],
    pub selected: usize,
}

impl Default for HotbarState {
    fn default() -> Self {
        Self {
            slots: [
                BlockId::Grass,
                BlockId::Dirt,
                BlockId::Stone,
                BlockId::Sand,
                BlockId::Wood,
                BlockId::Leaves,
                BlockId::CoalOre,
                BlockId::Water,
                BlockId::Grass,
            ],
            selected: 0,
        }
    }
}

impl HotbarState {
    pub fn selected_block(&self) -> BlockId {
        self.slots[self.selected]
    }
}

pub fn raycast_world(world: &WorldState, origin: Vec3, direction: Vec3, max_distance: f32) -> Option<RaycastHit> {
    if direction.length_squared() <= f32::EPSILON {
        return None;
    }

    let dir = direction.normalize();
    let mut distance = 0.0;
    let mut previous_voxel = origin.floor().as_ivec3();

    while distance <= max_distance {
        let point = origin + dir * distance;
        let current_voxel = point.floor().as_ivec3();

        let block = world.get_block_world(current_voxel);
        if block != BlockId::Air {
            let delta = current_voxel - previous_voxel;
            let hit_normal = if delta == IVec3::ZERO { IVec3::Y } else { -delta };
            return Some(RaycastHit {
                block_pos: current_voxel,
                hit_normal,
            });
        }

        previous_voxel = current_voxel;
        distance += RAYCAST_STEP;
    }

    None
}

pub fn update_selected_block_target(
    world: Res<WorldState>,
    camera_query: Query<&Transform, With<FpsCamera>>,
    mut selected: ResMut<SelectedBlockTarget>,
) {
    let Ok(camera) = camera_query.single() else {
        selected.0 = None;
        return;
    };

    let origin = camera.translation;
    let direction = *camera.forward();
    selected.0 = raycast_world(&world, origin, direction, RAYCAST_MAX_DISTANCE);
}

pub fn select_hotbar_slot(keyboard: Res<ButtonInput<KeyCode>>, mut hotbar: ResMut<HotbarState>) {
    let keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
    ];

    for (index, key) in keys.into_iter().enumerate() {
        if keyboard.just_pressed(key) {
            hotbar.selected = index;
            break;
        }
    }
}

pub fn break_and_place_blocks(
    mouse: Res<ButtonInput<MouseButton>>,
    cursor_options_query: Query<&CursorOptions, With<PrimaryWindow>>,
    selected_target: Res<SelectedBlockTarget>,
    hotbar: Res<HotbarState>,
    mut world: ResMut<WorldState>,
) {
    let Ok(cursor_options) = cursor_options_query.single() else {
        return;
    };

    if cursor_options.grab_mode != CursorGrabMode::Locked {
        return;
    }

    let Some(hit) = selected_target.0 else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        let block = world.get_block_world(hit.block_pos);
        if block.is_breakable() {
            let _ = world.set_block_world(hit.block_pos, BlockId::Air);
        }
    }

    if mouse.just_pressed(MouseButton::Right) {
        let place_pos = hit.block_pos + hit.hit_normal;
        if world.get_block_world(place_pos) == BlockId::Air {
            let _ = world.set_block_world(place_pos, hotbar.selected_block());
        }
    }
}

#[cfg(test)]
mod test;