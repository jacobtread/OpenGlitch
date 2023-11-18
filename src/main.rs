use crate::constants::{WINDOW_DEFAULT_HEIGHT, WINDOW_DEFAULT_WIDTH};
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    window::{WindowResolution, WindowTheme},
};
use bevy_framepace::{FramepacePlugin, FramepaceSettings};
use components::video::{VideoPlayer, VideoPlugin, VideoResource};
use constants::VERSION;

pub mod components;
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
        .add_plugins(FramepacePlugin)
        .insert_resource(FramepaceSettings {
            limiter: bevy_framepace::Limiter::from_framerate(30.),
        })
        .add_plugins(VideoPlugin)
        .add_systems(Startup, init_startup_movie)
        .run();
}

/// Plays the startup movie
fn init_startup_movie(
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    mut video_resource: NonSendMut<VideoResource>,
    asset_server: Res<AssetServer>,
) {
    const INTRO_MOVIE_FILE: &str = "data/Movies/xb_Loading2.bik";

    let (video_player, video_player_non_send) =
        VideoPlayer::new(INTRO_MOVIE_FILE, true, images).unwrap();

    commands.spawn(Camera2dBundle::default());
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            let entity = parent
                .spawn(ImageBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        ..default()
                    },
                    image: video_player.image_handle.clone().into(),
                    ..default()
                })
                .insert(video_player)
                .id();
            video_resource.data.insert(entity, video_player_non_send);
        });

    commands.spawn(AudioBundle {
        source: asset_server.load("../data/Movies/xb_intro$.wav"),
        ..Default::default()
    });
}
