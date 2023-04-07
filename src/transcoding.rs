use gstreamer as gst;
use gst::prelude::*;
// use gstreamer::{MessageView};
use std::error::Error as StdError;
use std::fmt;
use std::io::sink;
use crate::config::Config;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug)]
struct Error {
    details: String
}

impl Error {
    fn new(msg: &str) -> Error {
        Error{details: msg.to_string()}
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.details
    }
}

pub trait Transcoder {
   fn transcode(&self, path: &str) -> Result<String, Box<dyn StdError>>;
   fn generate_index_file(&self, path: &str) -> Result<(), Box<dyn StdError>>;
}

pub struct VideoTranscoder {

}

impl VideoTranscoder {
    pub fn build() -> Result<VideoTranscoder, &'static str> {
        Ok(VideoTranscoder {})
    }
}

impl Transcoder for VideoTranscoder {
     fn transcode(&self, video_uri: &str) -> Result<String, Box<dyn StdError>>{
        gst::init().unwrap();

        let filesrc = gst::ElementFactory::make("filesrc").build()?;
        let mpegtsmux = gst::ElementFactory::make("mpegtsmux").build()?;
        let filesink = gst::ElementFactory::make("hlssink2").build()?;
        let decodebin = gst::ElementFactory::make("decodebin").build()?;
        let x264enc = gst::ElementFactory::make("x264enc").build()?;
        let h264parse = gst::ElementFactory::make("h264parse").build()?;
        let avenc_acc = gst::ElementFactory::make("avenc_aac").build()?;
        let audioconvert = gst::ElementFactory::make("audioconvert").build()?;
        let video_queue = gst::ElementFactory::make("queue").build()?;
        let audio_queue = gst::ElementFactory::make("queue").build()?;
        let tee = gst::ElementFactory::make("tee").build()?;
        let tee_audio = gst::ElementFactory::make("tee").build()?;
        let tee_audio_2 = gst::ElementFactory::make("tee").build()?;
        let tee_audio_3 = gst::ElementFactory::make("tee").build()?;

        let queue_480p = gst::ElementFactory::make("queue").build()?;
        let videoconvert_480 = gst::ElementFactory::make("videoconvert").build()?;
        let videoscale_480p = gst::ElementFactory::make("videoscale").build()?;
        let x264enc_480p = gst::ElementFactory::make("x264enc").build()?;
        let mpegtsmux_480p = gst::ElementFactory::make("mpegtsmux").build()?;
        let hlssink_480p = gst::ElementFactory::make("hlssink2").build()?;
        let audio_queue_480 = gst::ElementFactory::make("queue").build()?;
        let audioconvert_480 = gst::ElementFactory::make("audioconvert").build()?;
        let avenc_acc_480 = gst::ElementFactory::make("avenc_aac").build()?;




        filesrc.set_property("location", video_uri);

        let split = video_uri.split(".");
        let vec: Vec<&str> = split.collect();
        let path_str = vec[0];

        let path = format!("{}_dir", path_str);

        std::fs::create_dir_all(&path);

        let target_duration: u32 = 6;
        let max_files: u32 = 10000000;

        filesink.set_property("location", format!("{}/original_%08d.ts", &path));
        filesink.set_property("playlist-location", format!("{}/video.m3u8", &path));
        filesink.set_property("target_duration", target_duration);
        filesink.set_property("max-files", max_files);

        hlssink_480p.set_property("location", format!("{}/480_%08d.ts", &path));
        hlssink_480p.set_property("playlist-location", format!("{}/video-480.m3u8", &path));
        hlssink_480p.set_property("target_duration", target_duration);
        hlssink_480p.set_property("max-files", max_files);


        let pipeline = gst::Pipeline::new(None);



        pipeline.add_many(&[
            &filesrc,
            &decodebin,
            &mpegtsmux,
            &filesink,
            &x264enc,
            &h264parse,
            &avenc_acc,
            &video_queue,
            &audio_queue,
            &queue_480p,
            &videoscale_480p,
            &x264enc_480p,
            &mpegtsmux_480p,
            &videoconvert_480,
            &hlssink_480p,
            &audioconvert,
            &tee,
            &tee_audio,
            &audio_queue_480,
            &audioconvert_480,
            &avenc_acc_480,
            &tee_audio_2,
            &tee_audio_3,
        ])?;

        filesrc.link(&decodebin)?;

        // Link Video
        tee.link(&video_queue)?;
        video_queue.link(&x264enc)?;
        x264enc.link(&filesink)?;


        let width = 640;
        let height = 480;
        let bitrate = 2000 * 1000;

        let caps_480 = gst::Caps::builder("video/x-raw")
            .field("width", width)
            .field("height", height)
            .build();

        tee.link(&queue_480p)?;
        queue_480p.link(&videoconvert_480)?;
        videoconvert_480.link(&videoscale_480p)?;
        videoscale_480p.link_filtered(&x264enc_480p, &caps_480)?;
        x264enc_480p.link(&hlssink_480p)?;


        // Link Audio
        tee_audio.link(&audio_queue)?;
        audio_queue.link(&audioconvert)?;
        audioconvert.link(&avenc_acc)?;
        avenc_acc.link(&tee_audio_2)?;
        tee_audio_2.link(&filesink)?;

        tee_audio.link(&audio_queue_480)?;
        audio_queue_480.link(&audioconvert_480)?;
        audioconvert_480.link(&avenc_acc_480)?;
        avenc_acc_480.link(&tee_audio_3)?;
        tee_audio_3.link(&hlssink_480p)?;

        // mpegtsmux.link(&filesink)?;

        decodebin.connect_pad_added(move |demux, src_pad|{
            println!("Received new pad {} from {}", src_pad.name(), demux.name());

            let new_pad_caps = src_pad.current_caps().unwrap();
            let new_pad_struct = new_pad_caps.structure(0).unwrap();
            let new_pad_type = new_pad_struct.name();

            println!("Pad type {}", new_pad_type);

            let new_pad_caps = src_pad.current_caps().unwrap();
            let new_pad_struct = new_pad_caps.structure(0).unwrap();
            let new_pad_type = new_pad_struct.name();

            if new_pad_type.starts_with("audio"){
                let sink_pad = tee_audio.static_pad("sink").unwrap();

                if sink_pad.is_linked(){
                    println!("Audio Pad already linked");
                    return;
                }

                let res = src_pad.link(&sink_pad);

                if res.is_err() {
                    println!("type of {} link failed: ", new_pad_type);
                }else {
                    println!("Linked successfully type {}:", new_pad_type)
                }
            } else if new_pad_type.starts_with("video"){
                let sink_pad = tee.static_pad("sink").unwrap();

                if sink_pad.is_linked(){
                    println!("video pad already linked!");
                    return;
                }

                let res = src_pad.link(&sink_pad);

                if res.is_err(){
                    println!("type of {} linked failed:", new_pad_type);
                }else {
                    println!("Linked succesfully type of {}:", new_pad_type);
                }
            }
        });

        decodebin.sync_state_with_parent()?;

        pipeline.set_state(gst::State::Playing)?;

        let bus = pipeline.bus().unwrap();

        for msg in bus.iter_timed(gst::ClockTime::NONE){
            use gst::MessageView;

            match msg.view(){
                MessageView::Error(err) => {
                    println!("Error received from element {:?} {}",
                             err.src().map(|s| s.path_string()),
                             err.error()
                    );
                    break;
                },
                MessageView::StateChanged(s) => {
                    println!(
                        "State changed from {:?}:{:?} -> {:?} ({:?})",
                        s.src().map(|s| s.path_string()),
                        s.old(),
                        s.current(),
                        s.pending()
                    );
                },
                MessageView::Eos(_) => break,
                _ => ()
            }
        }

        pipeline.set_state(gst::State::Null)?;

        self.generate_index_file(&path)?;

        Ok(path.to_owned())
    }

    fn generate_index_file(&self, path: &str) -> Result<(), Box<dyn StdError>> {

        let path_480 = "video-480.m3u8";
        let path_original = "video.m3u8";
        let mut index_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(format!("./{}/index.m3u8", &path))?;

        writeln!(index_file, "{}\n", "#EXTM3U")?;
        writeln!(index_file, "{}\n", "#EXT-X-STREAM-INF:BANDWIDTH=2000000,RESOLUTION=853x480")?;
        writeln!(index_file, "{}\n", path_480)?;
        writeln!(index_file, "{}\n", "#EXT-X-STREAM-INF:BANDWIDTH=5000000,RESOLUTION=1920x1080")?;
        writeln!(index_file, "{}\n", path_original)?;



        Ok(())
    }
}

