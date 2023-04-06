use std::borrow::Borrow;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter};
use tokio::fs::File;
use std::io::{Error, Write};
use tokio::io::{AsyncBufRead, AsyncReadExt, AsyncWriteExt, BufReader, AsyncRead};
use async_trait::async_trait;
use s3::Bucket;
use s3::creds::Credentials;
use s3::Region::Custom;
use crate::filestore::FileStoreResult::{Failed, Success};


#[derive(Debug)]
pub enum FileStoreResult {
    Success,
    Failed,
}

#[derive(Debug)]
pub struct FileFetchError{}

impl Display for FileFetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message not received")
    }
}

impl StdError for FileFetchError{}

#[async_trait]
pub trait FileStore: Send + Sync{
    async fn put_object(&self, buffer_reader: BufReader<File>, path: &str) -> Result<FileStoreResult, Box<dyn StdError>>;
    async fn get_object(&self, file_name: &str, file_uri: &str) -> Result<String, Box<dyn StdError>>;
}

pub struct Config {
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub region: Option<String>,
    pub bucket_name: Option<String>,
    pub endpoint: Option<String>
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
impl FileStore for S3Store {
    async fn put_object(&self, mut buffer_reader: BufReader<File>, path: &str) -> Result<FileStoreResult, Box<dyn StdError>> {
        // let mut reader = BufReader::new(file);
        let mut buffer = Vec::new();

        buffer_reader.read_to_end(&mut buffer).await?;

        let response_data = self.bucket.put_object(path, &buffer).await?;

        if response_data.status_code() != 200 {
           return Ok(Failed)
        }


        return Ok(Success);
    }

    async fn get_object(&self, file_name: &str,  file_uri: &str) -> Result<String, Box<dyn StdError>> {
        let path: String =  file_name.to_string();

        let response_data = self.bucket.get_object(file_uri).await?;

        if response_data.status_code() != 200 {
            return Err(FileFetchError{}.into());
        }

        //TODO: Save file and return path
        let mut video_file = File::create(file_name).await?;
        video_file.write_all(response_data.bytes()).await?;

        path.to_owned().push_str(file_name);

        Ok(path)
    }
}
