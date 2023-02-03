use std::borrow::Borrow;
use std::error::Error;
use tokio::io::{AsyncBufRead, AsyncReadExt, BufReader, AsyncRead};
use async_trait::async_trait;
use s3::{Bucket};
use s3::creds::Credentials;
use s3::Region::Custom;
use crate::filestore::FileStoreResult::{Failed, Success};




#[derive(Debug)]
pub enum FileStoreResult {
    Success,
    Failed,
}

#[async_trait]
pub trait FileStore<R: AsyncRead + Unpin + Send>{
    async fn put_object(&self, buffer_reader: BufReader<R>, path: &str) -> Result<FileStoreResult, Box<dyn Error>>;
    fn get_object(&self) -> Result<(), Box<dyn Error>>;
}

pub struct Config {
    pub(crate) access_key: Option<String>,
    pub(crate) secret_key: Option<String>,
    pub(crate) region: Option<String>,
    pub(crate) bucket_name: Option<String>,
    pub(crate) endpoint: Option<String>
}


pub struct S3Store {
    pub bucket: Bucket
}

impl S3Store {
    pub fn build(config: Config) -> Option<S3Store>{

        let credentials = Credentials{
            access_key: config.access_key,
            secret_key: config.secret_key,
            security_token: None,
            session_token: None,
            expiration: None,
        };

        let region = Custom {
            region: config.region.expect("region not provided"),
            endpoint: config.endpoint.expect("endpoint not provided")
        };

        let bucket = Bucket::new(
            config.bucket_name.as_deref().expect("bucket name not provided"),
            region,
            credentials
        ).expect("bucket initialization failed");

        Some(S3Store{
            bucket
        })
    }
}

#[async_trait]
impl<R: AsyncRead + Unpin + Send + 'static> FileStore<R> for S3Store {
    async fn put_object(&self, mut buffer_reader: BufReader<R>, path: &str) -> Result<FileStoreResult, Box<dyn Error>> {
        // let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();

        buffer_reader.read_to_end(&mut buffer).await?;

        let response_data = self.bucket.put_object(path, &buffer).await?;

        if response_data.status_code() != 200 {
           return Ok(Failed)
        }


        return Ok(Success);
    }

    fn get_object(&self) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}