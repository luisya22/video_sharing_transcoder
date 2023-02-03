
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Config {

    /// S3 access key
    #[arg(short, long)]
    pub access_key: String,

    /// S3 endpoint uri
    #[arg(short, long)]
    pub endpoint_uri: String,

    /// S3 secret key
    #[arg(short, long)]
    pub secret_access_key: String,

    /// S3 region
    #[arg(short, long)]
    pub region: String,

    /// S3 bucket name
    #[arg(short, long)]
    pub bucket_name: String,
}