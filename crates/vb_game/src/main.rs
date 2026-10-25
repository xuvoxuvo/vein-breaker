mod dev;
mod fx;
mod hud;
mod level;
mod mining;
mod player;
mod shared;
mod viewmodel;
mod weapon;

use avian3d::prelude::*;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Vein Breaker — FPS Prototype".to_string(),
                    ..default()
                }),
                ..default()
            }),
            PhysicsPlugins::default(),
        ))
        .insert_resource(ClearColor(Color::srgb(0.015, 0.017, 0.022)))
        .add_plugins((
            level::LevelPlugin,
            player::PlayerPlugin,
            fx::FxPlugin,
            viewmodel::ViewModelPlugin,
            weapon::WeaponPlugin,
            mining::MiningPlugin,
            hud::HudPlugin,
            dev::DevPlugin,
        ))
        .run();
}
