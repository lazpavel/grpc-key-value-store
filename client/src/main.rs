use kvstore::key_value_store_client::KeyValueStoreClient;
use lambda_extension::{
    service_fn,
    tracing::{self, info},
    Error, LambdaEvent, NextEvent,
};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::sync::Notify;
use tonic::transport::Channel;

mod handlers;
mod routes;

async fn async_work(client: Arc<Mutex<KeyValueStoreClient<Channel>>>, server_started: Arc<Notify>) {
    info!("starting http extension server...");
    let routes = routes::cache_routes(client);
    let (_, server) = warp::serve(routes).bind_ephemeral(([127, 0, 0, 1], 8888));
    server_started.notify_one(); // Notify that the server has started
    server.await
}

async fn extension(event: LambdaEvent) -> Result<(), Error> {
    info!("received event: {:?}", event.next);
    match event.next {
        NextEvent::Shutdown(_e) => {
            info!("server task completed, returning");
            Ok(())
        }
        NextEvent::Invoke(_e) => {
            info!("server task completed, returning");
            Ok(())
        }
    }
}

pub mod kvstore {
    tonic::include_proto!("kvstore");
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let start = Instant::now();
    info!("starting extension...");
    let client = Arc::new(Mutex::new(KeyValueStoreClient::connect("http://[::1]:8080").await?));
    let server_started = Arc::new(Notify::new());
    let server_future = async_work(client, server_started.clone());
    let _ = tokio::spawn(server_future);
    server_started.notified().await;
    info!("server started...");
    info!("latency L4 (Cold Start) {:?}", start.elapsed());

    // required to enable CloudWatch error logging by the runtime
    tracing::init_default_subscriber();
    lambda_extension::run(service_fn(|event| extension(event))).await
}
