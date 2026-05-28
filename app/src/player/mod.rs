use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::world::WorldState;

const MOVE_SPEED: f32 = 10.0;
const LOOK_SENSITIVITY: f32 = 0.002;
const GRAVITY: f32 = 25.0;
const JUMP_SPEED: f32 = 9.0;
pub const PLAYER_RADIUS: f32 = 0.30;
pub const PLAYER_HEIGHT: f32 = 1.8;
pub const EYE_HEIGHT: f32 = 1.62;

#[derive(Component)]
pub struct PlayerPhysics {
    pub velocity_y: f32,
    pub on_ground: bool,
}

impl Default for PlayerPhysics {
    fn default() -> Self {
        Self {
            velocity_y: 0.0,
            on_ground: false,
        }
    }
}

#[derive(Component)]
pub struct FpsCamera {
    pub yaw: f32,
    pub pitch: f32,
}

pub fn lock_cursor(
    mouse: Res<ButtonInput<MouseButton>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Ok(mut cursor_options) = cursor_options_query.single_mut() else {
        return;
    };

    if mouse.just_pressed(MouseButton::Left) {
        cursor_options.grab_mode = CursorGrabMode::Locked;
        cursor_options.visible = false;
    }

    if keyboard.just_pressed(KeyCode::Escape) {
        cursor_options.grab_mode = CursorGrabMode::None;
        cursor_options.visible = true;
    }
}

pub fn fps_look(
    mut mouse_motion: MessageReader<MouseMotion>,
    cursor_options_query: Query<&CursorOptions, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut Transform, &mut FpsCamera)>,
) {
    let Ok(cursor_options) = cursor_options_query.single() else {
        return;
    };

    if cursor_options.grab_mode != CursorGrabMode::Locked {
        mouse_motion.clear();
        return;
    }

    let mut delta = Vec2::ZERO;
    for motion in mouse_motion.read() {
        delta += motion.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    let Ok((mut transform, mut camera)) = camera_query.single_mut() else {
        return;
    };

    camera.yaw -= delta.x * LOOK_SENSITIVITY;
    camera.pitch -= delta.y * LOOK_SENSITIVITY;
    camera.pitch = camera.pitch.clamp(-1.54, 1.54);

    transform.rotation = Quat::from_axis_angle(Vec3::Y, camera.yaw)
        * Quat::from_axis_angle(Vec3::X, camera.pitch);
}

pub fn fps_move(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    world: Res<WorldState>,
    mut camera_query: Query<(&mut Transform, &mut PlayerPhysics), With<FpsCamera>>,
) {
    let Ok((mut transform, mut physics)) = camera_query.single_mut() else {
        return;
    };

    let mut axis = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        axis.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        axis.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        axis.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        axis.x += 1.0;
    }

    if keyboard.just_pressed(KeyCode::Space) && physics.on_ground {
        physics.velocity_y = JUMP_SPEED;
        physics.on_ground = false;
    }

    let forward = transform.forward();
    let forward_xz = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
    let right = transform.right();
    let right_xz = Vec3::new(right.x, 0.0, right.z).normalize_or_zero();

    let mut horizontal = Vec3::ZERO;
    if axis.length_squared() > 0.0 {
        horizontal = (forward_xz * -axis.y + right_xz * axis.x).normalize() * MOVE_SPEED * time.delta_secs();
    }

    physics.velocity_y -= GRAVITY * time.delta_secs();
    let vertical = physics.velocity_y * time.delta_secs();

    physics.on_ground = false;

    let mut position = transform.translation;
    position.x += horizontal.x;
    if collides_with_world(&world, position) {
        position.x -= horizontal.x;
    }

    position.z += horizontal.z;
    if collides_with_world(&world, position) {
        position.z -= horizontal.z;
    }

    position.y += vertical;
    if collides_with_world(&world, position) {
        position.y -= vertical;
        if vertical < 0.0 {
            physics.on_ground = true;
        }
        physics.velocity_y = 0.0;
    }

    transform.translation = position;
}

pub fn collides_with_blocks<F>(camera_position: Vec3, mut is_solid: F) -> bool
where
    F: FnMut(IVec3) -> bool,
{
    let min = camera_position + Vec3::new(-PLAYER_RADIUS, -EYE_HEIGHT, -PLAYER_RADIUS);
    let max = camera_position + Vec3::new(PLAYER_RADIUS, PLAYER_HEIGHT - EYE_HEIGHT, PLAYER_RADIUS);

    let min_i = min.floor().as_ivec3();
    let max_i = max.floor().as_ivec3();

    for x in min_i.x..=max_i.x {
        for y in min_i.y..=max_i.y {
            for z in min_i.z..=max_i.z {
                if is_solid(IVec3::new(x, y, z)) {
                    return true;
                }
            }
        }
    }

    false
}

fn collides_with_world(world: &WorldState, camera_position: Vec3) -> bool {
    collides_with_blocks(camera_position, |pos| world.get_block_world(pos).is_solid())
}

#[cfg(test)]
mod test;
