use avian3d::prelude::*;
use bevy::prelude::*;
use vb_core::BrickKind;

use crate::fx::{FxQueue, HitMark, Tracer};
use crate::level::Brick;
use crate::mining::OreSpawnQueue;
use crate::player::{Player, PlayerCamera, Recoil};
use crate::shared::{BrickMaterials, BricksLeft};

pub struct WeaponPlugin;

#[derive(Resource)]
pub struct Weapon {
    pub cooldown: f32,
    pub damage: i32,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            cooldown: 0.0,
            damage: 1,
        }
    }
}

const FIRE_INTERVAL: f32 = 0.11;
const MAX_RANGE: f32 = 120.0;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Weapon>().add_systems(Update, shoot);
    }
}

#[allow(clippy::too_many_arguments)]
fn shoot(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut weapon: ResMut<Weapon>,
    mut recoil: ResMut<Recoil>,
    mut left: ResMut<BricksLeft>,
    mut ore_queue: ResMut<OreSpawnQueue>,
    mut fx: ResMut<FxQueue>,
    camera: Query<&Transform, With<PlayerCamera>>,
    player: Query<Entity, With<Player>>,
    spatial: SpatialQuery,
    mut bricks: Query<(Entity, &mut Brick, &Transform), Without<PlayerCamera>>,
    materials: Res<BrickMaterials>,
    mut commands: Commands,
) {
    weapon.cooldown = (weapon.cooldown - time.delta_secs()).max(0.0);

    if !buttons.pressed(MouseButton::Left) || weapon.cooldown > 0.0 {
        return;
    }

    let (Ok(camera), Ok(player_entity)) = (camera.single(), player.single()) else {
        return;
    };

    weapon.cooldown = FIRE_INTERVAL;
    recoil.0 = (recoil.0 + 0.012).min(0.06);

    let origin = camera.translation;
    let direction = camera.forward();
    let filter = SpatialQueryFilter::default().with_excluded_entities([player_entity]);

    let end = if let Some(hit) = spatial.cast_ray(origin, direction, MAX_RANGE, true, &filter) {
        let point = origin + direction * hit.distance;
        let mut color = Color::srgb(1.0, 0.55, 0.2);

        if let Ok((entity, mut brick, transform)) = bricks.get_mut(hit.entity) {
            color = match brick.kind {
                BrickKind::Copper => Color::srgb(1.0, 0.55, 0.2),
                BrickKind::Iron => Color::srgb(0.75, 0.85, 1.0),
                BrickKind::Rock => Color::srgb(0.8, 0.8, 0.75),
                BrickKind::Bedrock => Color::srgb(0.35, 0.38, 0.45),
            };

            let remaining = vb_core::apply_damage(brick.hp, weapon.damage);
            brick.hp = remaining;

            if remaining == 0 {
                ore_queue.0.push((transform.translation, brick.value));
                left.0 = (left.0 - 1).max(0);
                commands.entity(entity).despawn();
            } else {
                let tier = (remaining.clamp(1, 3) - 1) as usize;
                commands
                    .entity(entity)
                    .insert(MeshMaterial3d(materials.tiers[tier].clone()));
            }
        }

        fx.hits.push(HitMark {
            pos: point,
            normal: hit.normal,
            life: 0.12,
            color,
        });

        point
    } else {
        origin + direction * MAX_RANGE
    };

    fx.tracers.push(Tracer {
        from: origin + camera.rotation * crate::viewmodel::muzzle_offset(),
        to: end,
        life: 0.07,
    });
}
