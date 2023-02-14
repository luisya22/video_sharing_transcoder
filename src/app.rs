use std::sync::Arc;
use deadpool_lapin::Pool;
use tokio::io::AsyncRead;
use crate::filestore::FileStore;
use crate::message_broker::MessageBroker;
use crate::video::Video;

pub struct App<R: AsyncRead + Unpin + Send> {
   pub processor: fn(Video),
   pub file_store: Arc<dyn FileStore<R> + Send + Sync>,
   pub message_broker: Arc<dyn MessageBroker>
}