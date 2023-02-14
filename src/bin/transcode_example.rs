
pub fn main(){
    let transcoder = video_sharing_transcoder::transcoding::Transcoder::build().unwrap();

    transcoder.transcode("backup.mp4").unwrap();
}
