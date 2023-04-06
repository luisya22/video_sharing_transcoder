use video_sharing_transcoder::directory_upload_manager::UploadManager;
use video_sharing_transcoder::filestore::Config;
use video_sharing_transcoder::filestore::S3Store;
use std::env;
use video_sharing_transcoder::directory_upload_manager::DirectoryUploadManager;
use std::sync::Arc;

#[tokio::main]
async fn main(){

    let file_store_endpoint = env::var("FILESTORE_ENDPOINT").unwrap();
    let access_key = env::var("FILESTORE_ACCESS_KEY").unwrap();
    let secret_access_key = env::var("FILESTORE_SECRET").unwrap();
    let region = env::var("FILESTORE_REGION").unwrap();
    let bucket_name = env::var("FILESTORE_BUCKET_NAME").unwrap();
    let dir = env::var("DIR").unwrap();

    let file_store_config = Config{
        access_key: Option::from(access_key),
        secret_key: Option::from(secret_access_key),
        region: Option::from(region),
        bucket_name: Option::from(bucket_name),
        endpoint: Option::from(file_store_endpoint),
    };

    let s3_store = S3Store::build(file_store_config);

    if let Some(s3) = s3_store {
        let filestore = Arc::new(s3);

        let directory_upload_manager: Arc<dyn UploadManager> = Arc::new(DirectoryUploadManager{});

        println!("Starting directory_upload_manager");

        directory_upload_manager.upload_directory(&dir, filestore).await.unwrap();
    } else {
        panic!("error: bucket not created");
    }
}
