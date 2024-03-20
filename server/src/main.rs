use kvstore::{
    key_value_store_server::{KeyValueStore, KeyValueStoreServer},
    KvGetRequest, KvResponse, KvSetRequest,
};
use std::sync::Mutex;
use redis::{Client, Connection, RedisError, Commands};
use tonic::{transport::Server, Request, Response, Status};

pub mod kvstore {
    tonic::include_proto!("kvstore");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::open("rediss://amplify-hosting-shared-cache-demo-0acerw.serverless.use1.cache.amazonaws.com:6379").unwrap();
    let connection = Mutex::new(client.get_connection()?);

    let address = "[::1]:8080".parse().unwrap();
    let service = KeyValueStoreService::new(connection);

    Server::builder()
        .add_service(KeyValueStoreServer::new(service))
        .serve(address)
        .await?;
    Ok(())
}

pub struct KeyValueStoreService {
    connection: Mutex<Connection>,
}

impl KeyValueStoreService {
    pub fn new(connection: Mutex<Connection>) -> Self {
        Self { connection }
    }
}

#[tonic::async_trait]
impl KeyValueStore for KeyValueStoreService {
    async fn set(&self, request: Request<KvSetRequest>) -> Result<Response<KvResponse>, Status> {
        let r = request.into_inner();
        println!("Received set request: {:?}", r);
        let mut connection = self.connection.lock().unwrap();
        let _: Result<String, RedisError> = connection.set(&r.key, &r.value);

        Ok(Response::new(KvResponse {
            status_code: 0,
            message: "Set request received".into(),
            value: "".into(),
        }))
    }

    async fn get(&self, request: Request<KvGetRequest>) -> Result<Response<KvResponse>, Status> {
        let r = request.into_inner();
        println!("Received get request: {:?}", r);
        let mut connection = self.connection.lock().unwrap();
        let result = connection.get(r.key);

        Ok(Response::new(KvResponse {
            status_code: 0,
            message: "Get request received".into(),
            value: result.unwrap(),
        }))
    }
}
