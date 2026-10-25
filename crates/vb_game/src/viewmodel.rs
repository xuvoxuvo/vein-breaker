use bevy::{color::LinearRgba, prelude::*};

use crate::player::PlayerCamera;

pub struct ViewModelPlugin;

/// A part of the first-person weapon, in gun-local space.
#[derive(Component)]
pub struct ViewModel {
    pub local: Vec3,
}

/// Where the gun sits relative to the camera, in camera-local space.
const GUN_ORIGIN: Vec3 = Vec3::new(0.23, -0.20, -0.40);
const GUN_SCALE: f32 = 0.6;

fn gun_rotation() -> Quat {
    // Slight inward yaw + downward pitch reads better than a perfectly
    // parallel weapon.
    Quat::from_euler(EulerRot::YXZ, 0.10, -0.05, 0.0)
}

/// Muzzle position in camera-local space (used to start tracers visually).
pub fn muzzle_offset() -> Vec3 {
    GUN_ORIGIN + gun_rotation() * (Vec3::new(0.0, 0.035, -0.62) * GUN_SCALE)
}

impl Plugin for ViewModelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_view_model)
            .add_systems(PostUpdate, follow_camera);
    }
}

fn spawn_view_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let body = meshes.add(Cuboid::new(0.13, 0.17, 0.40));
    let barrel = meshes.add(Cuboid::new(0.055, 0.055, 0.34));
    let muzzle = meshes.add(Cuboid::new(0.085, 0.085, 0.10));
    let magazine = meshes.add(Cuboid::new(0.10, 0.17, 0.11));
    let grip = meshes.add(Cuboid::new(0.08, 0.19, 0.10));
    let scope = meshes.add(Cuboid::new(0.05, 0.05, 0.14));
    let strip = meshes.add(Cuboid::new(0.015, 0.03, 0.30));

    let metal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.13, 0.15),
        metallic: 0.85,
        perceptual_roughness: 0.35,
        emissive: LinearRgba::rgb(0.012, 0.014, 0.018),
        ..default()
    });
    let accent = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.45, 0.10),
        emissive: LinearRgba::rgb(1.2, 0.35, 0.03),
        perceptual_roughness: 0.5,
        ..default()
    });

    let parts = [
        (body, metal.clone(), Vec3::ZERO),
        (barrel, metal.clone(), Vec3::new(0.0, 0.035, -0.36)),
        (muzzle, metal.clone(), Vec3::new(0.0, 0.035, -0.56)),
        (magazine, metal.clone(), Vec3::new(0.0, -0.14, 0.02)),
        (grip, metal.clone(), Vec3::new(0.0, -0.15, 0.17)),
        (scope, metal, Vec3::new(0.0, 0.13, 0.02)),
        (strip, accent, Vec3::new(0.072, 0.045, -0.20)),
    ];

    for (mesh, material, local) in parts {
        commands.spawn((
            ViewModel { local },
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_translation(local),
        ));
    }
}

fn follow_camera(
    camera: Query<&Transform, With<PlayerCamera>>,
    mut parts: Query<(&ViewModel, &mut Transform), Without<PlayerCamera>>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let gun_rotation = gun_rotation();
    let origin = camera.translation + camera.rotation * GUN_ORIGIN;

    for (part, mut transform) in &mut parts {
        transform.translation =
            origin + camera.rotation * (gun_rotation * (part.local * GUN_SCALE));
        transform.rotation = camera.rotation * gun_rotation;
        transform.scale = Vec3::splat(GUN_SCALE);
    }
}
