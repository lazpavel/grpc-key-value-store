use kvstore::{
    key_value_store_server::{KeyValueStore, KeyValueStoreServer},
    KvGetRequest, KvResponse, KvSetRequest,
};
use std::sync::{Arc, Mutex};
use redis::{aio::ConnectionManager, AsyncCommands, Client, RedisError};
use tonic::{transport::Server, Request, Response, Status};

pub mod kvstore {
    tonic::include_proto!("kvstore");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Proxy server listening on port 8080...");
    let addr = redis::ConnectionAddr::TcpTls {
        host: "amplify-hosting-shared-cache-demo-0acerw.serverless.use1.cache.amazonaws.com".to_string(),
        port: 6379,
        insecure: true,
        tls_params: None
    };

    let conn_info = redis::ConnectionInfo {
        addr,
        redis: redis::RedisConnectionInfo {
            db: 0,
            username: None,
            password: None,
        }
    };

    let client = Client::open(conn_info).unwrap();
    // let client = Client::open("redis://127.0.0.1:6379").unwrap();
    let manager = ConnectionManager::new(client).await?;
    let shared_manager = Arc::new(Mutex::new(manager));
    let address = "[::]:8080".parse().unwrap();
    let service = KeyValueStoreService::new(shared_manager);

    Server::builder()
        .add_service(KeyValueStoreServer::new(service))
        .serve(address)
        .await?;
    Ok(())
}

pub struct KeyValueStoreService {
    manager: Arc<Mutex<ConnectionManager>>,
}

impl KeyValueStoreService {
    pub fn new(manager: Arc<Mutex<ConnectionManager>>) -> Self {
        Self { manager }
    }
}

#[tonic::async_trait]
impl KeyValueStore for KeyValueStoreService {
    async fn set(&self, request: Request<KvSetRequest>) -> Result<Response<KvResponse>, Status> {
        let r = request.into_inner();
        let mut manager = self.manager.lock().unwrap().clone();
        let _: Result<String, RedisError> = manager.set(&r.key, &r.value).await;

        Ok(Response::new(KvResponse {
            status_code: 0,
            message: "Set request received".into(),
            value: "".into(),
        }))
    }

    async fn get(&self, request: Request<KvGetRequest>) -> Result<Response<KvResponse>, Status> {
        let r = request.into_inner();
        let mut manager = self.manager.lock().unwrap().clone();
        let result = manager.get(r.key).await;

        Ok(Response::new(KvResponse {
            status_code: 0,
            message: "Get request received".into(),
            value: result.unwrap(),
        }))
    }
}
