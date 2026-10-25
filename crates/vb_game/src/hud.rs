use bevy::prelude::*;

use crate::shared::{BricksLeft, Inventory};

pub struct HudPlugin;

#[derive(Component)]
struct HudText;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, update_hud);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.86, 0.66)),
        Node {
            position_type: PositionType::Absolute,
            top: px(18),
            left: px(18),
            ..default()
        },
        HudText,
    ));

    commands.spawn((
        Text::new("LMB FIRE   WASD MOVE   SPACE JUMP   R REBUILD   F1 CURSOR   ESC QUIT"),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::srgb(0.55, 0.55, 0.5)),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(16),
            left: px(18),
            ..default()
        },
    ));

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: px(4),
            height: px(4),
            ..default()
        },
        BackgroundColor(Color::srgb(1.0, 0.9, 0.62)),
    ));
}

fn update_hud(
    inventory: Res<Inventory>,
    bricks: Res<BricksLeft>,
    mut text: Query<&mut Text, With<HudText>>,
) {
    if !inventory.is_changed() && !bricks.is_changed() {
        return;
    }
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    text.0 = format!("ORE {:03}    BLOCKS {:02}", inventory.ore, bricks.0);
}
