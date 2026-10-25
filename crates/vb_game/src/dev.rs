use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

pub struct DevPlugin;

#[derive(Resource)]
struct ScreenshotMode {
    path: String,
}

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, quit_on_escape);

        let args: Vec<String> = std::env::args().collect();
        if let Some(index) = args.iter().position(|arg| arg == "--screenshot") {
            let path = args
                .get(index + 1)
                .cloned()
                .unwrap_or_else(|| "screenshot.png".to_string());
            app.insert_resource(ScreenshotMode { path });
            app.add_systems(Update, take_screenshot);
        }
    }
}

fn quit_on_escape(keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

fn take_screenshot(mode: Res<ScreenshotMode>, mut commands: Commands, mut frame: Local<u32>) {
    *frame += 1;

    if *frame == 180 {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(mode.path.clone()));
    }

    if *frame == 300 {
        std::process::exit(0);
    }
}
