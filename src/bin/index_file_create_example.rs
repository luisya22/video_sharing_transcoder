use video_sharing_transcoder::transcoding::{VideoTranscoder, Transcoder};



#[tokio::main]
async fn main() {

    let transcoder = VideoTranscoder::build().unwrap();

    transcoder.generate_index_file("video1").unwrap();
}
