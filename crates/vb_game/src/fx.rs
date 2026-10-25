use bevy::prelude::*;

/// Very short-lived visual feedback that is cheaper to draw as gizmos than as entities.
#[derive(Resource, Default)]
pub struct FxQueue {
    pub tracers: Vec<Tracer>,
    pub hits: Vec<HitMark>,
}

pub struct Tracer {
    pub from: Vec3,
    pub to: Vec3,
    pub life: f32,
}

pub struct HitMark {
    pub pos: Vec3,
    pub normal: Vec3,
    pub life: f32,
    pub color: Color,
}

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FxQueue>().add_systems(Update, draw_fx);
    }
}

fn draw_fx(mut fx: ResMut<FxQueue>, time: Res<Time>, mut gizmos: Gizmos) {
    let dt = time.delta_secs();

    for tracer in &mut fx.tracers {
        gizmos.line(tracer.from, tracer.to, Color::srgb(1.0, 0.82, 0.45));
        tracer.life -= dt;
    }

    for hit in &mut fx.hits {
        gizmos.line(hit.pos, hit.pos + hit.normal * 0.35, hit.color);
        hit.life -= dt;
    }

    fx.tracers.retain(|t| t.life > 0.0);
    fx.hits.retain(|h| h.life > 0.0);
}
