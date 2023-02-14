extern crate video_sharing_transcoder;

use std::error::Error;

#[tokio::main]
async fn main() {
    
    //TODO: Main should pass args
    let result = video_sharing_transcoder::run().await;

    match result {
        Ok(_) => println!("Everything is good"),
        Err(_) => eprintln!("It Failed")
    }

}
