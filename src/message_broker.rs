use std::borrow::Borrow;
use std::io::Read;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use deadpool_lapin::{Manager, Pool, PoolError};
use futures::{AsyncBufRead, StreamExt};
use lapin::options::{BasicAckOptions, BasicConsumeOptions, QueueDeclareOptions};
use lapin::types::FieldTable;
use thiserror::Error as ThisError;
use crate::filestore::FileStore;
use lapin::ConnectionProperties;
use tokio_amqp::LapinTokioExt;

type RMQResult<T> = Result<T, PoolError>;
type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("rmq error: {0}")]
    RMQError(#[from] lapin::Error),
    #[error("rmq pool error: {0}")]
    RMQPoolError(#[from] PoolError)
}

#[async_trait]
pub trait MessageBroker {
   async fn listen(&self) -> Result<(), Error>;
}

pub struct RabbitMq {
    pub pool: Pool,
    pub queue_name: String,
    pub processor: fn(&str, Arc<dyn FileStore + Send + Sync>),
    pub file_store: Arc<dyn FileStore + Send + Sync>
}

impl RabbitMq {
    async fn get_rmq_con(&self) ->RMQResult<Connection> {
        let connection = self.pool.get().await?;

        Ok(connection)
    }
    async fn init_rmq_listen(&self) -> Result<(), Error>{
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

                let s = match std::str::from_utf8(&data){
                    Ok(v) => v,
                    _ => ""
                };

                println!("data str: {:?}", s);


                (self.processor)(s, self.file_store.clone());

                channel
                    .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
                    .await?;
            } else {
               println!("No messages");
            }
        }


        Ok(())
    }
}

#[async_trait]
impl MessageBroker for RabbitMq {
    async fn listen(&self) -> Result<(), Error>{
        println!("I'm Here");
       let mut retry_interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            retry_interval.tick().await;
            println!("connection rmq consumer...");

            match self.init_rmq_listen().await {
                Ok(_) => println!("rmq listen returned"),
                Err(e) => eprintln!("rmq listen had an error: {}", e)
            }
        }
    }
}
