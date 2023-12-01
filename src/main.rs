use std::{
    fs::{read_to_string, File},
    os::windows::fs::MetadataExt,
};

use crate::constants::{WINDOW_DEFAULT_HEIGHT, WINDOW_DEFAULT_WIDTH};
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    render::render_resource::PrimitiveTopology,
    window::{WindowResolution, WindowTheme},
};
use bevy_flycam::prelude::*;
use bevy_framepace::{FramepacePlugin, FramepaceSettings};
use binrw::BinRead;
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
        .add_plugins(VideoPlugin)
        // .add_systems(Startup, init_startup_movie)
        .add_systems(Startup, init_startup_mesh_test)
        .add_plugins(PlayerPlugin)
        .run();
}

fn init_startup_mesh_test(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mut buffer = read_to_string("data/buffer_dump.txt").unwrap();
    let values: Vec<[f32; 3]> = buffer
        .lines()
        .filter_map(|line| {
            let mut split = line.splitn(3, ',');
            let a = split.next()?;
            let b = split.next()?;
            let c = split.next()?;

            let a: f32 = a.parse().ok()?;
            let b: f32 = b.parse().ok()?;
            let c: f32 = c.parse().ok()?;

            Some((a * 2.0, b * 2.0, c * 2.0))
        })
        .map(|(a, b, c)| [a, b, c])
        .collect();

    let mut buffer = read_to_string("data/buffer_dump_index.txt").unwrap();
    let indicies: Vec<u16> = buffer
        .lines()
        .filter_map(|line| {
            let a: u16 = line.trim().parse().ok()?;

            Some(a)
        })
        .collect();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleStrip)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, values)
        .with_indices(Some(bevy::render::mesh::Indices::U16(indicies)));

    // let mut file = File::open("data/ape/gcdggltch00.ape").unwrap();
    // let mut header: FMesh = FMesh::read(&mut file).unwrap();
    // println!("Length: {}", file.metadata().unwrap().file_size());
    // // dbg!(&header);
    // let mut mesh_data = (header.mesh_data.value.take()).unwrap();
    // let vb = mesh_data
    //     .vertex_buffers
    //     .value
    //     .take()
    //     .unwrap()
    //     .pop()
    //     .unwrap();
    // let mesh = create_bevy_mesh(vb);
    let handle = meshes.add(mesh);

    // Render the mesh with the custom texture using a PbrBundle, add the marker.
    commands.spawn((PbrBundle {
        mesh: handle,

        ..default()
    },));
}

/// Plays the startup movie
fn init_startup_movie(
    mut commands: Commands,
    images: ResMut<Assets<Image>>,
    mut video_resource: NonSendMut<VideoResource>,
    asset_server: Res<AssetServer>,
) {
    const INTRO_MOVIE_FILE: &str = "data/Movies/xb_intro$.bik";

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
