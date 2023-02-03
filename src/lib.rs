use std::env;
use std::error::Error;
use std::sync::Arc;
use deadpool_lapin::{Manager, Pool};
use lapin::ConnectionProperties;
use tokio::fs::File;
use tokio_amqp::LapinTokioExt;
use crate::filestore::{FileStore, S3Store};
use crate::message_broker::{MessageBroker, RabbitMq};
use crate::video::Video;

mod transcoding;
mod config;
mod filestore;
mod message_broker;
mod video;


pub async fn run() -> Result<(), Box<dyn Error>>{
    let pool_amount_str = env::var("POOL_QUANTITY")
        .unwrap_or_else(|_| "1".into());

    let pool_amount: usize = pool_amount_str.parse()?;

    let addr = env::var("AMQP_ADDR")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2f".into());

   let queue_name = env::var("QUEUE_NAME")
       .unwrap_or_else(|_| "video_queue".into());

    let manager = Manager::new(addr, ConnectionProperties::default().with_tokio());
    let pool: Pool = Pool::builder(manager)
        .max_size(pool_amount)
        .build()?;

    let filestore_endpoint = env::var("FILESTORE_ENDPOINT")?;
    let access_key = env::var("FILESTORE_ACCESS_KEY")?;
    let secret_access_key = env::var("FILESTORE_SECRET")?;
    let region = env::var("FILESTORE_REGION")?;
    let bucket_name = env::var("FILESTORE_BUCKET_NAME")?;


    let file_store_config = filestore::Config{
        access_key: Option::from(access_key),
        secret_key: Option::from(secret_access_key),
        region: Option::from(region),
        bucket_name: Option::from(bucket_name),
        endpoint: Option::from(filestore_endpoint),
    };

    let s3_store = S3Store::build(file_store_config);

    if let Some(s3) = s3_store {
        let message_broker = RabbitMq{
            pool,
            queue_name,
            processor: process_message as fn(&Video, Arc<dyn FileStore<File> + Send + Sync>),
            file_store: Arc::new(s3)
        };

        message_broker.listen().await?;
    } else {
        panic!("error: bucket not created")
    }

    Ok(())
}

pub fn process_message(data: &Video, filestore: Arc<dyn FileStore<File> + Send + Sync>){
    println!("This is the message: {:?}", data);
}


