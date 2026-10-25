use avian3d::prelude::*;
use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

/// First-person look angles, in radians. Yaw 0 looks down -Z.
#[derive(Component, Default)]
pub struct Look {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Component)]
pub struct MoveTuning {
    pub speed: f32,
    pub jump_speed: f32,
}

/// Camera kick, decayed every frame. Added to pitch while rendering.
#[derive(Resource, Default)]
pub struct Recoil(pub f32);

const MOUSE_SENSITIVITY: f32 = 0.0022;
const EYE_HEIGHT: f32 = 1.0;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Recoil>()
            .add_systems(Startup, (spawn_player, grab_cursor).chain())
            .add_systems(Update, (look, toggle_cursor, decay_recoil))
            .add_systems(FixedUpdate, (walk, jump).chain())
            .add_systems(PostUpdate, sync_camera);
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Look::default(),
        MoveTuning {
            speed: 6.0,
            jump_speed: 6.0,
        },
        RigidBody::Dynamic,
        Collider::capsule(0.4, 1.0),
        LockedAxes::ROTATION_LOCKED,
        LinearDamping(0.4),
        Friction::new(0.0),
        Restitution::new(0.0),
        GravityScale(1.0),
        Transform::from_xyz(0.0, 1.1, 8.0),
    ));

    commands.spawn((
        PlayerCamera,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 75f32.to_radians(),
            ..default()
        }),
        AmbientLight {
            color: Color::srgb(0.42, 0.48, 0.62),
            brightness: 420.0,
            ..default()
        },
        Transform::from_xyz(0.0, 1.85, 8.0),
    ));
}

fn grab_cursor(mut cursor: Query<&mut CursorOptions>, mut done: Local<bool>) {
    if *done {
        return;
    }
    let Ok(mut cursor) = cursor.single_mut() else {
        return;
    };
    cursor.grab_mode = CursorGrabMode::Locked;
    cursor.visible = false;
    *done = true;
}

fn look(mouse: Res<AccumulatedMouseMotion>, mut player: Query<&mut Look, With<Player>>) {
    let Ok(mut look) = player.single_mut() else {
        return;
    };
    look.yaw -= mouse.delta.x * MOUSE_SENSITIVITY;
    look.pitch = (look.pitch - mouse.delta.y * MOUSE_SENSITIVITY).clamp(-1.45, 1.45);
}

fn walk(
    keys: Res<ButtonInput<KeyCode>>,
    mut player: Query<(&mut LinearVelocity, &Look, &MoveTuning), With<Player>>,
) {
    let Ok((mut velocity, look, tuning)) = player.single_mut() else {
        return;
    };

    let mut direction = Vec3::ZERO;
    let forward = Vec3::new(-look.yaw.sin(), 0.0, -look.yaw.cos());
    let right = Vec3::new(look.yaw.cos(), 0.0, -look.yaw.sin());

    if keys.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= right;
    }

    let target = direction.normalize_or_zero() * tuning.speed;
    velocity.x += (target.x - velocity.x) * 0.35;
    velocity.z += (target.z - velocity.z) * 0.35;
}

fn jump(
    keys: Res<ButtonInput<KeyCode>>,
    spatial: SpatialQuery,
    mut player: Query<(Entity, &mut LinearVelocity, &Transform, &MoveTuning), With<Player>>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    let Ok((entity, mut velocity, transform, tuning)) = player.single_mut() else {
        return;
    };

    let filter = SpatialQueryFilter::default().with_excluded_entities([entity]);
    let grounded = spatial
        .cast_ray(transform.translation, Dir3::NEG_Y, 1.05, true, &filter)
        .is_some();

    if grounded {
        velocity.y = tuning.jump_speed;
    }
}

fn decay_recoil(mut recoil: ResMut<Recoil>) {
    recoil.0 *= 0.88;
}

// The disjoint filters are required to avoid a B0001 query conflict.
#[allow(clippy::type_complexity)]
fn sync_camera(
    player: Query<(&Transform, &Look), (With<Player>, Without<PlayerCamera>)>,
    mut camera: Query<&mut Transform, With<PlayerCamera>>,
    recoil: Res<Recoil>,
) {
    let Ok((body, look)) = player.single() else {
        return;
    };
    let Ok(mut camera) = camera.single_mut() else {
        return;
    };

    camera.translation = body.translation + Vec3::Y * EYE_HEIGHT;
    camera.rotation = Quat::from_euler(EulerRot::YXZ, look.yaw, look.pitch + recoil.0, 0.0);
}

fn toggle_cursor(keys: Res<ButtonInput<KeyCode>>, mut cursor: Query<&mut CursorOptions>) {
    if !keys.just_pressed(KeyCode::F1) {
        return;
    }
    let Ok(mut cursor) = cursor.single_mut() else {
        return;
    };
    let locked = cursor.grab_mode == CursorGrabMode::Locked;
    cursor.grab_mode = if locked {
        CursorGrabMode::None
    } else {
        CursorGrabMode::Locked
    };
    cursor.visible = locked;
}
