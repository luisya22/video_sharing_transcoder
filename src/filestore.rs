use std::borrow::Borrow;
use std::error::Error;
use tokio::io::{AsyncBufRead, AsyncReadExt, BufReader};
use async_trait::async_trait;
use futures::AsyncRead;
use s3::{Bucket};
use s3::creds::Credentials;
use s3::Region::Custom;
use tokio::fs::File;
use crate::filestore::FileStoreResult::{Failed, Success};




#[derive(Debug)]
pub enum FileStoreResult {
    Success,
    Failed,
}

#[async_trait]
pub trait FileStore{
    async fn put_object(&self, buffer_reader: BufReader<File>, path: &str) -> Result<FileStoreResult, Box<dyn Error>>;
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
impl FileStore for S3Store {
    async fn put_object(&self, mut buffer_reader: BufReader<File>, path: &str) -> Result<FileStoreResult, Box<dyn Error>> {
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

// async fn backup() {
//     let args = Config::parse();
//
//     // let transcoder = transcoding::Transcoder::build(args).unwrap();
//
//
//     // transcoder.transcode("videos/fat_bunny.mp4").expect("Transcoder did not worked");
//
//     // Create S3 Bucket
//     let s3Config = Config{
//         access_key: Option::from(args.access_key),
//         secret_key: Option::from(args.secret_access_key),
//         region: Option::from(args.region),
//         bucket_name: Option::from(args.bucket_name),
//         endpoint: Option::from(args.endpoint_uri),
//     };
//
//     let bucket = S3Store::build(s3Config);
//     let mut file = File::open("fat_bunny.mp4");
//     let mut buffer_reader = BufReader::new(file.unwrap());
//
//     if let Some(b) = bucket {
//         // filestore::put_object(&b.bucket, buffer_reader, "/videos/fat_bunny.mp4").await.expect("Error Uploading");
//
//         let(head_object_result, code) = b.bucket.head_object("/videos/fat_bunny.mp4").await.expect("Error");
//         println!("{:?}", head_object_result);
//
//         let response_data = b.bucket.get_object("/videos/fat_bunny.mp4").await.expect("Error");
//
//         b.get_object()
//
//
//
//         let mut file = File::create("video.mp4").expect("Error creating file");
//         file.write_all(response_data.bytes()).expect("Error passing content to file");
//     } else {
//         panic!("Bucket not created")
//     }
// }