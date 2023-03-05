use std::{error::Error, sync::Arc};
use async_trait::async_trait;
use tokio::{fs::File, io::{BufReader, AsyncRead}};

use crate::filestore::{FileStore, FileStoreResult};

#[async_trait]
pub trait UploadManager<R: AsyncRead + Unpin + Send> {
    async fn upload_directory(&self, path: &str, filestore: Arc<dyn FileStore<Arc<R>>>) -> Result<(), Box<dyn Error>>;
}

pub struct DirectoryUploadManager {}

#[async_trait]
impl<R: AsyncRead + Unpin + Send> UploadManager<R> for DirectoryUploadManager{
    async fn upload_directory(&self, path: &str, filestore: Arc<dyn FileStore<Arc<File>>>) -> Result<(), Box<dyn Error>>{

        let mut reader = tokio::fs::read_dir(path).await?;

        // path = directory path

        loop{
            if let Some(f) = reader.next_entry().await? {
                println!("hello");


                let file = File::open(f.path()).await?;
                let buf_reader = BufReader::new(file); 
                // let split: Vec<&str> = path.split(".").collect();
                // let path = split[0];

                // file path now i need 
                //
                let filename = f.file_name().to_string_lossy().into_owned();

                let mut filepath: String = path.to_owned();
                filepath.push_str(&filename);


                let result = filestore.put_object(buf_reader, path).await?;

                if let FileStoreResult::Success = result {
                    println!("Error uploading file");
                }

                match result {
                    FileStoreResult::Failed => println!("Error uploading file"),
                    FileStoreResult::Success => {}
                }

            } else {
                break;
            }
        }

        Ok(())
    } 
}


