extern crate cpal;
use cpal::{StreamData, UnknownTypeOutputBuffer};
use cpal::traits::{DeviceTrait, HostTrait, EventLoopTrait};
use minimp3::Decoder;
use std::fs::File;
use minimp3::Frame;
use minimp3::Error;
use std::path::Path;
use std::sync::mpsc::channel;

fn main() {

    let mut decoder = Decoder::new(File::open(
        Path::new("res/dnxk.mp3")
    ).unwrap());//1559457768

    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to get default output device");
    let format = device.default_output_format().expect("Failed to get default output format");
    let event_loop = host.event_loop();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id.clone());

    let mut data = Vec::new();
    let mut frame_channel = 0;
    let speed = 1;
    loop {
        match decoder.next_frame() {
            Ok(mut f) => {
                /*song_channels = f.channels;
                song_sample_rate = f.sample_rate;*/
                for sample in f.data.chunks_mut(f.channels as usize) {
                    for a in sample.iter_mut() {
                        data.push(a.clone());
                    }
                }
                if frame_channel == 0 {
                    frame_channel = f.channels
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    let mut current_frame_data_index = 0;
    let mut next_value = || {
        let s = match current_frame_data_index < data.len() {
            true => {
                current_frame_data_index += (frame_channel/frame_channel * speed) as usize;
                data[current_frame_data_index]
            },
            false => {
                0i16
            }
        };
        s as f32 / std::i16::MAX as f32// as f32 / std::i16::MAX as f32
    };
    event_loop.run(move |stream_id, stream_result| {
        let stream_data = match stream_result {
             Ok(data) => data,
             Err(err) => {
                 eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                 return;
             }
             _ => {
                 println!("empty data");
                 return;
             },
         };
        match stream_data {
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                println!("1");
                for sample in buffer.chunks_mut(format.channels as usize) {
                    for out in sample.iter_mut() {
                        *out = 0;
                    }
                }
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                println!("2");
                for elem in buffer.iter_mut() {
                    *elem = 0;
                }
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    for out in sample.iter_mut() {
                        *out = next_value();
                    }
                }
            },
            _ => (),
        }
    });
}