use gstreamer as gst;
use gst::prelude::*;
// use gstreamer::{MessageView};
use std::error::Error as StdError;
use std::fmt;
use crate::config::Config;

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

pub struct Transcoder {
    access_key: String,
    endpoint_uri: String,
    secret_access_key: String,
}

impl Transcoder {
    pub fn build(args: Config) -> Result<Transcoder, &'static str> {
        let access_key = args.access_key;
        let endpoint_uri = args.endpoint_uri;
        let secret_access_key =  args.secret_access_key;

        Ok(Transcoder{
            access_key,
            endpoint_uri,
            secret_access_key,
        })
    }

    pub fn transcode(&self, video_uri: &str) -> Result<String, Box<dyn StdError>>{
        let str_arr = video_uri.split("/").collect::<Vec<&str>>();

        if str_arr.len() < 2 {
            return Err(Box::new(Error::new("video uri wrong format")))
        }

        gst::init().unwrap();

        let filesrc = gst::ElementFactory::make("awss3src").build()?;
        let qtmux = gst::ElementFactory::make("qtmux").build()?;
        let filesink = gst::ElementFactory::make("hlssink2").build()?;
        let decodebin = gst::ElementFactory::make("decodebin").build()?;
        let x264enc = gst::ElementFactory::make("x264enc").build()?;
        let avenc_acc = gst::ElementFactory::make("avenc_aac").build()?;
        let video_queue = gst::ElementFactory::make("queue").build()?;
        let audio_queue = gst::ElementFactory::make("queue").build()?;

        filesrc.set_property("access_key", &self.access_key);
        filesrc.set_property("endpoint_uir", &self.endpoint_uri);
        filesrc.set_property("secret_access_key", &self.secret_access_key);
        filesrc.set_property("uri", &video_uri);

        filesink.set_property("location", str_arr[1]);

        let pipeline = gst::Pipeline::new(None);

        pipeline.add_many(&[
            &filesrc,
            &decodebin,
            &qtmux,
            &filesink,
            &x264enc,
            &avenc_acc,
            &video_queue,
            &audio_queue
        ])?;

        filesrc.link(&decodebin)?;

        // Link Video
        video_queue.link(&x264enc)?;
        x264enc.link(&qtmux)?;

        // Link Audio
        audio_queue.link(&avenc_acc)?;
        avenc_acc.link(&qtmux)?;


        qtmux.link(&filesink)?;

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
                let sink_pad = audio_queue.static_pad("sink").unwrap();

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
                let sink_pad = video_queue.static_pad("sink").unwrap();

                if sink_pad.is_linked(){
                    println!("video pad already linked!");
                    return;
                }

                let res = src_pad.link(&sink_pad);

                if res.is_err(){
                    println!("type of {} linked faile:", new_pad_type);
                }else {
                    println!("Linked succesfully type of {}:", new_pad_type)
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

        Ok("".to_string())
    }
}

