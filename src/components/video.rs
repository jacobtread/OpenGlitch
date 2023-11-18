use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::utils::hashbrown::HashMap;
use ffmpeg_next::format::{input, Pixel};
use ffmpeg_next::frame::Video;
use ffmpeg_next::software::scaling::context::Context as ScalingContext;
use ffmpeg_next::software::scaling::Flags;
use std::path::Path;

/// Resource for storing internal video player data which is !Send
#[derive(Default)]
pub struct VideoResource {
    /// Mapping between spawned video player entities and their data
    pub data: HashMap<Entity, VideoPlayerInternal>,
}

pub struct VideoPlugin;

impl Plugin for VideoPlugin {
    fn build(&self, app: &mut App) {
        app.init_non_send_resource::<VideoResource>();
        app.add_systems(Startup, init_ffmpeg);
        app.add_systems(Update, play_video);
    }
}

/// Video player data
pub struct VideoPlayerInternal {
    input_context: ffmpeg_next::format::context::Input,
    decoder: ffmpeg_next::decoder::Video,
    scaler: ffmpeg_next::software::scaling::Context,
    stream_index: usize,
}

#[derive(Component)]
pub struct VideoPlayer {
    pub image_handle: Handle<Image>,
}

impl VideoPlayer {
    pub fn new<P>(
        path: P,
        mut images: ResMut<Assets<Image>>,
    ) -> Result<(VideoPlayer, VideoPlayerInternal), ffmpeg_next::Error>
    where
        P: AsRef<Path>,
    {
        let input = input(&path)?;
        let video_stream = input
            .streams()
            .best(ffmpeg_next::media::Type::Video)
            .ok_or(ffmpeg_next::Error::StreamNotFound)?;
        let stream_index = video_stream.index();

        let context_decoder =
            ffmpeg_next::codec::context::Context::from_parameters(video_stream.parameters())?;
        let decoder = context_decoder.decoder().video()?;

        let scaler = ScalingContext::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGBA,
            decoder.width(),
            decoder.height(),
            Flags::BILINEAR,
        )?;

        let mut image = Image::new_fill(
            Extent3d {
                width: decoder.width(),
                height: decoder.height(),
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &Color::BLACK.as_rgba_u8(),
            TextureFormat::Rgba8UnormSrgb,
        );
        image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;

        let image_handle: Handle<Image> = images.add(image);

        Ok((
            VideoPlayer { image_handle },
            VideoPlayerInternal {
                input_context: input,
                decoder,
                scaler,
                stream_index,
            },
        ))
    }
}

/// System that initialized ffmpeg
fn init_ffmpeg() {
    ffmpeg_next::init().expect("Failed to initialize FFmpeg");
}

/// System that handles decoding video frames and displaying them onto
/// the video player image resource
fn play_video(
    mut video_player_query: Query<(&mut VideoPlayer, Entity)>,
    mut video_resource: NonSendMut<VideoResource>,
    mut images: ResMut<Assets<Image>>,
) {
    for (video_player, entity) in video_player_query.iter_mut() {
        let data = video_resource.data.get_mut(&entity).unwrap();
        // read packets from stream until complete frame received
        while let Some((stream, packet)) = data.input_context.packets().next() {
            // check if packets is for the selected video stream
            if stream.index() == data.stream_index {
                // pass packet to decoder
                data.decoder.send_packet(&packet).unwrap();
                let mut decoded = Video::empty();
                // check if complete frame was received
                if data.decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = Video::empty();
                    // run frame through scaler for color space conversion
                    data.scaler.run(&decoded, &mut rgb_frame).unwrap();
                    // update data of image texture
                    let image = images.get_mut(&video_player.image_handle).unwrap();
                    image.data.copy_from_slice(rgb_frame.data(0));
                    return;
                }
            }
        }
        // no frame received
        // signal end of playback to decoder
        match data.decoder.send_eof() {
            Err(ffmpeg_next::Error::Eof) => {}
            other => other.unwrap(),
        }
    }
}
