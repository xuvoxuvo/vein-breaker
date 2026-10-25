use bevy::prelude::*;

use crate::player::Player;
use crate::shared::{FxAssets, Inventory};

pub struct MiningPlugin;

/// Blocks push ore spawns here; the mining systems own the actual entities.
#[derive(Resource, Default)]
pub struct OreSpawnQueue(pub Vec<(Vec3, u32)>);

#[derive(Component)]
pub struct Ore {
    pub value: u32,
}

#[derive(Component, Default)]
pub struct OreMotion {
    pub velocity: Vec3,
    pub age: f32,
}

impl Plugin for MiningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OreSpawnQueue>()
            .add_systems(Update, (spawn_ore, collect_ore).chain());
    }
}

fn spawn_ore(
    mut commands: Commands,
    mut queue: ResMut<OreSpawnQueue>,
    assets: Res<FxAssets>,
    mut seed: Local<u32>,
) {
    for (position, value) in queue.0.drain(..) {
        *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let angle = (*seed % 628) as f32 / 100.0;

        commands.spawn((
            Ore { value },
            OreMotion {
                velocity: Vec3::new(angle.cos() * 2.2, 3.2, angle.sin() * 2.2),
                age: 0.0,
            },
            Mesh3d(assets.ore_mesh.clone()),
            MeshMaterial3d(assets.ore_material.clone()),
            Transform::from_translation(position),
        ));
    }
}

fn collect_ore(
    time: Res<Time>,
    mut inventory: ResMut<Inventory>,
    player: Query<&Transform, With<Player>>,
    mut ore: Query<(Entity, &mut Transform, &mut OreMotion, &Ore), Without<Player>>,
    mut commands: Commands,
) {
    let Ok(player) = player.single() else {
        return;
    };
    let dt = time.delta_secs();

    for (entity, mut transform, mut motion, ore) in &mut ore {
        motion.age += dt;

        if motion.age < 0.4 {
            motion.velocity.y -= 14.0 * dt;
            transform.translation += motion.velocity * dt;

            if transform.translation.y < 0.25 {
                transform.translation.y = 0.25;
                motion.velocity.y = 0.0;
                motion.velocity.x *= 0.85;
                motion.velocity.z *= 0.85;
            }
        } else {
            let target = player.translation + Vec3::Y * 0.4;
            let delta = target - transform.translation;
            let distance = delta.length();

            if distance < 0.7 {
                inventory.ore += ore.value;
                commands.entity(entity).despawn();
            } else {
                transform.translation += delta / distance * (12.0 * dt).min(distance);
            }
        }
    }
}
