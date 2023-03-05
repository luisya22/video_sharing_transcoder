use std::borrow::Borrow;
use std::fmt::{Display, Formatter, write};
use std::io::Read;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use deadpool_lapin::{Manager, Pool, PoolError};
use futures::StreamExt;
use lapin::options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use thiserror::Error as ThisError;
use crate::filestore::FileStore;
use lapin::ConnectionProperties;
use tokio::io::AsyncRead;
use tokio_amqp::LapinTokioExt;
use crate::video::Video;
use serde_json::Error as SerdeJsonError;
use crate::app::App;

type RMQResult<T> = Result<T, PoolError>;
type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;

#[derive(Debug)]
pub struct NotMessageError;

impl Display for NotMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Message not received")
    }
}

impl std::error::Error for NotMessageError {}

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("rmq error: {0}")]
    Rmq(#[from] lapin::Error),

    #[error("rmq pool error: {0}")]
    RMQPool(#[from] PoolError),

    #[error("serde json error: {0}")]
    SerdeJson(#[from] SerdeJsonError),

    #[error("not message error: {0}")]
    NotMessage(#[from] NotMessageError)
}

#[async_trait]
pub trait MessageBroker{
   async fn listen(&self) -> Result<(), Error>;
}

pub struct RabbitMq<R: AsyncRead + Unpin + Send>{
    pub pool: Pool,
    pub queue_name: String,
    pub app: App<R>
}

impl<R: AsyncRead + Unpin + Send> RabbitMq<R>{
    async fn get_rmq_con(&self) ->RMQResult<Connection> {
        let connection = self.pool.get().await?;

        Ok(connection)
    }
    async fn init_rmq_listen(&self) -> Result<Video, Error>{
        let rmq_con = self.get_rmq_con().await.map_err(|e| {
          eprintln!("could not get rmq con: {}", e);
          e
        })?;

        let channel = rmq_con.create_channel().await?;

        let options = QueueDeclareOptions{
            passive: false,
            durable: true,
            exclusive: false,
            auto_delete: false,
            nowait: false,
        };

        let queue = channel
            .queue_declare(
                &self.queue_name,
                options,
                FieldTable::default()
            ).await?;

        let mut consumer = channel
            .basic_consume(
                queue.name().borrow(),
                "",
                BasicConsumeOptions::default(),
                FieldTable::default()
            )
            .await?;

        while let Some(delivery) = consumer.next().await {
            if let Ok((channel, delivery)) = delivery {
                println!("received msg: {:?}", delivery);

                let data = delivery.data;

                let video_data: Video = serde_json::from_slice(&data)?;

                channel
                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                    .await?;

                return Ok(video_data);
            } else {
               println!("No messages");
            }
        }

        Err(Error::NotMessage(NotMessageError))
    }
}

#[async_trait]
impl<R: AsyncRead + Unpin + Send> MessageBroker for RabbitMq<R> {
    async fn listen(&self) -> Result<(), Error>{
        println!("I'm Here");
       let mut retry_interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            retry_interval.tick().await;
            println!("connection rmq consumer...");

            match self.init_rmq_listen().await {
                Ok(video_data) => {
                    self.app.process_message(video_data);
                },
                Err(e) => eprintln!("rmq listen had an error: {}", e)
            }
        }
    }
}
