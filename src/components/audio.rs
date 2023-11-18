// /// FFmpeg audio source from video playback
// pub struct FAudio {
//     pub path: PathBuf,
// }

// impl Decodable for FAudio {
//     type DecoderItem = f32;
//     type Decoder = FAudioDecoder;

//     fn decoder(&self) -> Self::Decoder {
//         let input_context = input(&self.path).unwrap();
//         let audio_stream = input_context
//             .streams()
//             .best(ffmpeg_next::media::Type::Audio)
//             .ok_or(ffmpeg_next::Error::StreamNotFound)
//             .unwrap();

//         let context_decoder =
//             ffmpeg_next::codec::context::Context::from_parameters(audio_stream.parameters())
//                 .unwrap();
//         let audio_decoder = context_decoder.decoder().audio().unwrap();
//         let audio_stream_index = audio_stream.index();

//         FAudioDecoder {
//             decoder: audio_decoder,
//             input_context,
//             audio_stream_index,
//             frame: None,
//             frame_index: 0,
//             frame_planes: 0,
//         }
//     }
// }

// pub struct FAudioDecoder {
//     /// ffmpeg video decoder
//     decoder: ffmpeg_next::decoder::Audio,
//     /// ffmpeg input context
//     input_context: ffmpeg_next::format::context::Input,
//     /// index of the audio stream
//     audio_stream_index: usize,

//     /// The current audio frame
//     frame: ffmpeg_next::frame::Audio,

//     frame_index: usize,
//     frame_planes: usize,
// }

// impl FAudioDecoder {
//     pub fn new() -> Result<Self, ffmpeg_next::Error> {
//         let input_context = input(&self.path).unwrap();
//         let audio_stream = input_context
//             .streams()
//             .best(ffmpeg_next::media::Type::Audio)
//             .ok_or(ffmpeg_next::Error::StreamNotFound)
//             .unwrap();

//         let context_decoder =
//             ffmpeg_next::codec::context::Context::from_parameters(audio_stream.parameters())
//                 .unwrap();
//         let audio_decoder = context_decoder.decoder().audio().unwrap();
//         let audio_stream_index = audio_stream.index();
//     }
// }

// impl Iterator for FAudioDecoder {
//     type Item = f32;

//     fn next(&mut self) -> Option<Self::Item> {
//         None
//     }
// }

// impl Source for FAudioDecoder {
//     fn current_frame_len(&self) -> Option<usize> {
//         None
//     }

//     fn channels(&self) -> u16 {
//         self.decoder.channels()
//     }

//     fn sample_rate(&self) -> u32 {
//         self.decoder.rate()
//     }

//     fn total_duration(&self) -> Option<std::time::Duration> {
//         None
//     }
// }
