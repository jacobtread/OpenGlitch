use crate::constants::{WINDOW_DEFAULT_HEIGHT, WINDOW_DEFAULT_WIDTH};
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    window::{WindowResolution, WindowTheme},
};
use constants::VERSION;

pub mod constants;
pub mod formats;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .build()
                // Custom window settings
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("OpenMA v{}", VERSION),
                        resolution: WindowResolution::new(
                            WINDOW_DEFAULT_WIDTH,
                            WINDOW_DEFAULT_HEIGHT,
                        ),
                        window_theme: Some(WindowTheme::Dark),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                // Update logging
                .set(LogPlugin {
                    level: Level::DEBUG,
                    filter: "wgpu=error,naga=warn,open_ma=debug,bevy_app=warn,bevy_render=warn"
                        .to_string(),
                }),
        )
        .run();
}
