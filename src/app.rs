use std::borrow::Borrow;
use tokio::fs::File;
use std::sync::Arc;
use deadpool_lapin::Pool;
use std::error::Error;
use async_trait::async_trait;
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, BufReader, AsyncRead};


use crate::filestore::FileStore;
use crate::message_broker::MessageBroker;
use crate::transcoding::Transcoder;
use crate::video::Video;
use crate::directory_upload_manager::UploadManager;

pub struct App<R: AsyncRead + Unpin + Send> {
    pub file_store: Arc<dyn FileStore<R> + Send + Sync>,
    pub transcoder: Arc<dyn Transcoder + Send + Sync>,
    pub directory_upload_manager: Arc<dyn UploadManager<R>>,
}

impl<R: AsyncRead + Unpin + Send> App<R> {
  pub async fn process_message(&self, data: Video) -> Result<(), Box<dyn Error>>{
    println!("This is the message: {:?}", data);

    // Download Video
    let file_path = self.file_store.get_object(data.name.borrow(), data.path.borrow()).await?;

    println!("{:?}", file_path.to_owned());
    
    // Transcode video
    let video_chunks_path = self.transcoder.transcode(file_path)?; 
    // Upload Video chunks and masterplaylist
    // TODO: Move this to another struct or function attached to App. App should only glue items
    // not do any work
    //
    let result = self.directory_upload_manager.upload_directory(&video_chunks_path, Arc::clone(&self.file_store)).await?;      

    match result {
        Ok(_) => println!("Video Uploaded"),
        Err(_) => println!("Video Upload Failed"),
    }

    // Send message to RabbitMq
    
    Ok(())

  }
}
