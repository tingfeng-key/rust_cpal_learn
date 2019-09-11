use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let mut audio = AudioPlayBack::new();
    audio.playback();
    audio.run("res/dnxk.mp3");
    thread::sleep(Duration::from_secs(80));
    audio.run("res/情歌王 - 古巨基.mp3");
    thread::park();

}
//控制器
struct AudioControl {
    speed: usize
}
//音频数据结构
struct Audio {
    control: AudioControl,
    source_path: Option<String>,
    current_source_data: Vec<i16>,
    current_frame_channel: usize,
    current_frame_data_index: usize
}

impl Audio {
    //实例
    pub fn new() -> Self {
        let control = AudioControl {
            speed: 1
        };
        Self {
            control,
            source_path: None,
            current_frame_channel: 0,
            current_source_data: Vec::new(),
            current_frame_data_index: 0
        }
    }
    //播放
    pub fn play(&mut self, source_path: &str) {
        self.source_path = Some(String::from(source_path));
        self.current_source_data = Vec::new();
        let data = self.decoder();
        self.current_frame_data_index = 0;
        self.current_source_data = data;
    }
    //解码
    fn decoder(&mut self) -> Vec<i16>{
        use minimp3::Decoder;
        use std::{fs::File, path::Path};

        println!("解码开始...");
        let source_path = self.source_path.clone().expect("source_path error");
        let source_file = File::open(
            Path::new(&source_path)
        ).expect("打开资源文件失败...");
        let mut decoder = Decoder::new(source_file);
        let mut data = Vec::new();
        loop {
            match decoder.next_frame() {
                Ok(mut f) => {
                    for sample in f.data.chunks_mut(f.channels as usize) {
                        for a in sample.iter_mut() {
                            data.push(a.clone());
                        }
                    }
                    if self.current_frame_channel == 0 {
                        self.current_frame_channel = f.channels
                    }
                }
                Err(_) => {
                    println!("解码完成...");
                    break;
                }
            }
        }
        data
    }
    //获取输出值
    fn get_next_value(&mut self) -> f32 {
        let s = match self.current_frame_data_index < self.current_source_data.len() {
            true => {
                let speed = self.control.speed.clone();
                let channel = self.current_frame_channel.clone();
                let value = self.current_source_data[self.current_frame_data_index];
                let inc_index = channel / channel * speed;
                self.current_frame_data_index += inc_index;
                value
            },
            false => {
                0i16
            }
        };
        s as f32 / std::i16::MAX as f32// as f32 / std::i16::MAX as f32
    }
}

//后台播放器
struct AudioPlayBack {
    audio: Arc<Mutex<Audio>>,
}

impl AudioPlayBack {
    pub fn new() -> Self {
        Self {
            audio: Arc::new(Mutex::new(Audio::new()))
        }
    }
    //播放音频
    pub fn run(&self, source_path: &str) {
        self.audio.lock().expect("play error").play(source_path);
    }
    //后台运行
    pub fn playback(&mut self) {
        extern crate cpal;
        use std::thread;
        use cpal::{
            StreamData::Output,
            UnknownTypeOutputBuffer::{U16, I16, F32},
            traits::{DeviceTrait, HostTrait, EventLoopTrait}
        };

        let host = cpal::default_host();
        let device = host.default_output_device().expect("Failed to get default output device");
        let format = device.default_output_format().expect("Failed to get default output format");
        let event_loop = host.event_loop();
        let stream_id = event_loop.build_output_stream(&device, &format).expect("build_output_stream error");
        event_loop.play_stream(stream_id.clone()).expect("play_stream error");
        let audio = self.audio.clone();
        thread::spawn(move || {
            event_loop.run(move |stream_id, stream_result| {
                let stream_data = match stream_result {
                    Ok(data) => data,
                    Err(err) => {
                        eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                        return;
                    }
                };
                match stream_data {
                    Output { buffer: U16(mut buffer) } => {
                        for sample in buffer.chunks_mut(format.channels as usize) {
                            for out in sample.iter_mut() {
                                *out = 0;
                            }
                        }
                    },
                    Output { buffer: I16(mut buffer) } => {
                        for elem in buffer.iter_mut() {
                            *elem = 0;
                        }
                    },
                    Output { buffer: F32(mut buffer) } => {
                        for sample in buffer.chunks_mut(format.channels as usize) {
                            for out in sample.iter_mut() {
                                *out = audio.lock().expect("get_next_value error").get_next_value();
                            }
                        }
                    },
                    _ => (),
                }
            });
        });
    }
}