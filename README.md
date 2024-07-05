# Video Sharing Transcoder

## Overview
The Video Sharing Transcoder service is responsible for transcoding video files and generating HLS playlists. It communicates with the [Video Sharing API](https://github.com/luisya22/video-sharing-go) to process video uploads and manage transcoding tasks. The transcoding process involves converting video files into multiple resolutions and creating HLS playlists.

## Features
- *Transcode Video:* Converts video files into diffferent resolutions and formats.
- *Generate Index FIle:* Creates an HLS index file for the transcoded videos.
- *Integration with RabbitMQ:* Listens for video upload messages and process them.

## Installation
1. Clone Repository
     ```sh
     git clone https://github.com/luisya22/video_sharing_transcoder.git
     cd video_sharing_transcoder
     ```

2. Create a `.env` file in the root directory and add the necessary configurations.
3. Build th eproject:
  ```sh
  cargo build --release
  ```

## Configuration
The application requires several environment variables to be set. These include configurajtions for the AWS S3 storage, RabbitMQ, and other essential settings.

Example `.env` file:
```env
FILESTORE_ENDPOINT=your_s3_endpoint
FILESTORE_ACCESS_KEY=your_aws_access_key_id
FILESTORE_SECRET=your_aws_secret_key
FILESTORE_REGION=your_aws_region
FILESTORE_BUCKET_NAME=your_aws_bucket_name

AMQP_ADDR=your_rabbitmq_url
QUEUE_NAME=video_queue
WRITE_QUEUE_NAME=write_video_queue
POOL_QUANTITY=1
```
## Usage
### Running the Service
1. Include environment variables
  ```sh
  source .env
  ```
2. Run the service:
  ```sh
  cargo run --release
  ```

## Message Processing
The service listens to RabbitMQ messages for new video uploads. When a message is received, it downloads the video file from S3, trancodes it, uploads the chunks back to S3, and sends a message with the path to the generated index file.

## License
This project is licensed under the MIT License.

---
For more details about the Video Sharing API, refer to its [repository](https://github.com/luisya22/video-sharing-go)

