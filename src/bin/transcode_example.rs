use video_sharing_transcoder::transcoding::Transcoder;

pub fn main(){
    let transcoder = video_sharing_transcoder::transcoding::VideoTranscoder::build().unwrap();

    transcoder.transcode("backup.mp4").unwrap();
}
