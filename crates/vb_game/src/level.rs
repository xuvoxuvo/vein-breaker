use avian3d::prelude::*;
use bevy::{color::LinearRgba, prelude::*};
use vb_core::BrickKind;

use crate::shared::{BrickMaterials, BricksLeft, FxAssets, Inventory};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inventory>()
            .init_resource::<BricksLeft>()
            .add_systems(Startup, setup_world)
            .add_systems(Update, restart_level);
    }
}

/// A destructible (or bedrock) mine block.
#[derive(Component)]
pub struct Brick {
    pub hp: i32,
    pub kind: BrickKind,
    pub value: u32,
}

/// Marker for anything that should be rebuilt when the player restarts.
#[derive(Component)]
pub struct Block;

#[derive(Resource)]
pub struct LevelAssets {
    pub cube: Handle<Mesh>,
}

const WALL_PATTERN: [&str; 6] = [
    "############",
    "#.CC....CC.#",
    "#.CC.CC.CC.#",
    "#....CC....#",
    "#.II.CC.II.#",
    "############",
];

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let ore_mesh = meshes.add(Cuboid::new(0.16, 0.16, 0.16));

    let bedrock = materials.add(StandardMaterial {
        base_color: Color::srgb(0.13, 0.135, 0.155),
        perceptual_roughness: 1.0,
        ..default()
    });
    let tier1 = materials.add(StandardMaterial {
        base_color: Color::srgb(0.30, 0.13, 0.06),
        perceptual_roughness: 0.9,
        ..default()
    });
    let tier2 = materials.add(StandardMaterial {
        base_color: Color::srgb(0.62, 0.26, 0.09),
        perceptual_roughness: 0.8,
        emissive: LinearRgba::rgb(0.06, 0.015, 0.0),
        ..default()
    });
    let tier3 = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.45, 0.14),
        perceptual_roughness: 0.7,
        emissive: LinearRgba::rgb(0.16, 0.045, 0.005),
        ..default()
    });
    let ore_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.55, 0.18),
        emissive: LinearRgba::rgb(0.55, 0.18, 0.02),
        perceptual_roughness: 0.4,
        ..default()
    });

    commands.insert_resource(FxAssets {
        ore_mesh,
        ore_material,
    });

    let mats = BrickMaterials {
        tiers: [tier1, tier2, tier3],
        bedrock: bedrock.clone(),
    };

    spawn_room(&mut commands, &cube, &bedrock);
    spawn_lights(&mut commands);

    let count = spawn_blocks(&mut commands, &cube, &mats);
    commands.insert_resource(LevelAssets { cube });
    commands.insert_resource(BricksLeft(count));
    commands.insert_resource(mats);
}

fn spawn_blocks(commands: &mut Commands, cube: &Handle<Mesh>, mats: &BrickMaterials) -> i32 {
    let mut destructible = 0;

    for (row, line) in WALL_PATTERN.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            let Some(kind) = BrickKind::from_char(ch) else {
                continue;
            };
            let position = Vec3::new(-5.5 + col as f32, 1.2 + row as f32, -11.45);
            spawn_brick(commands, cube, mats, position, kind);
            if kind.destructible() {
                destructible += 1;
            }
        }
    }

    for (side, x) in [(-1.0f32, -11.45f32), (1.0, 11.45)] {
        for row in 0..3 {
            for col in 0..2 {
                let position = Vec3::new(x, 3.5 + row as f32, -5.5 + col as f32);
                // Side veins are iron on the left, copper on the right.
                let kind = if side < 0.0 {
                    BrickKind::Iron
                } else {
                    BrickKind::Copper
                };
                spawn_brick(commands, cube, mats, position, kind);
                destructible += 1;
            }
        }
    }

    destructible
}

fn spawn_room(commands: &mut Commands, cube: &Handle<Mesh>, rock: &Handle<StandardMaterial>) {
    let blocks = [
        (Vec3::new(0.0, -0.5, 0.0), Vec3::new(26.0, 1.0, 26.0)),
        (Vec3::new(0.0, 9.5, 0.0), Vec3::new(26.0, 1.0, 26.0)),
        (Vec3::new(-12.5, 4.5, 0.0), Vec3::new(1.0, 9.0, 26.0)),
        (Vec3::new(12.5, 4.5, 0.0), Vec3::new(1.0, 9.0, 26.0)),
        (Vec3::new(0.0, 4.5, -12.5), Vec3::new(26.0, 9.0, 1.0)),
        (Vec3::new(0.0, 4.5, 12.5), Vec3::new(26.0, 9.0, 1.0)),
    ];

    for (position, size) in blocks {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(rock.clone()),
            Transform::from_translation(position).with_scale(size),
            RigidBody::Static,
            Collider::cuboid(size.x * 0.5, size.y * 0.5, size.z * 0.5),
        ));
    }
}

fn spawn_brick(
    commands: &mut Commands,
    cube: &Handle<Mesh>,
    mats: &BrickMaterials,
    position: Vec3,
    kind: BrickKind,
) {
    if !kind.destructible() {
        commands.spawn((
            Block,
            Mesh3d(cube.clone()),
            MeshMaterial3d(mats.bedrock.clone()),
            Transform::from_translation(position).with_scale(Vec3::splat(0.98)),
            RigidBody::Static,
            Collider::cuboid(0.51, 0.51, 0.51),
        ));
        return;
    }

    let hp = kind.max_hp();
    let tier = (hp.clamp(1, 3) - 1) as usize;

    commands.spawn((
        Block,
        Brick {
            hp,
            kind,
            value: kind.ore_value(),
        },
        Mesh3d(cube.clone()),
        MeshMaterial3d(mats.tiers[tier].clone()),
        Transform::from_translation(position).with_scale(Vec3::splat(0.94)),
        RigidBody::Static,
        Collider::cuboid(0.51, 0.51, 0.51),
    ));
}

fn spawn_lights(commands: &mut Commands) {
    for (x, z) in [(-6.0, -6.0), (6.0, -6.0), (0.0, 5.0)] {
        commands.spawn((
            PointLight {
                color: Color::srgb(1.0, 0.72, 0.42),
                intensity: 1_400_000.0,
                range: 26.0,
                ..default()
            },
            Transform::from_xyz(x, 8.4, z),
        ));
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 700.0,
            ..default()
        },
        Transform::from_xyz(6.0, 12.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn restart_level(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    blocks: Query<Entity, With<Block>>,
    assets: Res<LevelAssets>,
    materials: Res<BrickMaterials>,
    mut left: ResMut<BricksLeft>,
    mut inventory: ResMut<Inventory>,
) {
    if !keys.just_pressed(KeyCode::KeyR) {
        return;
    }

    for entity in &blocks {
        commands.entity(entity).despawn();
    }

    let count = spawn_blocks(&mut commands, &assets.cube, &materials);
    left.0 = count;
    inventory.ore = 0;
}
