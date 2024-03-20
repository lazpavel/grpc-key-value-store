use kvstore::{key_value_store_client::KeyValueStoreClient, KvGetRequest, KvSetRequest};
use lambda_extension::tracing::info;
use std::{collections::HashMap, convert::Infallible, sync::Arc, sync::Mutex, time::Instant};
use warp::http::StatusCode;
use tonic::transport::Channel;

use crate::kvstore;

pub async fn get_handler(
    key: String,
    client: Arc<Mutex<KeyValueStoreClient<Channel>>>,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let get_request = tonic::Request::new(KvGetRequest {
        key,
    });
    let start = Instant::now();
    let mut client = {
        let lock = client.lock().unwrap();
        lock.clone()
    };
    let get_response = client.get(get_request).await;
    info!("latency L3 (get) {:?}", start.elapsed());

    match get_response {
        Ok(get_response) => {
            let kv_response = get_response.into_inner();
            Ok(Box::new(warp::reply::json(&kv_response.value)))
        },
        Err(e) => {
            info!("get response message: {}", e);
            Ok(Box::new(warp::reply::json(&"{}")))
        },
    }
}

pub async fn set_handler(
    body: HashMap<String, String>,
    client: Arc<Mutex<KeyValueStoreClient<Channel>>>,
) -> Result<impl warp::Reply, Infallible> {
    let key = body.get("key").unwrap().to_string();
    let data = body.get("data").unwrap().to_string();
    let set_request = tonic::Request::new(KvSetRequest {
        key,
        value: data,
    });
    let start = Instant::now();
    let mut client = {
        let lock = client.lock().unwrap();
        lock.clone()
    };
    let _ = client.set(set_request).await;
    info!("latency L3 (set) {:?}", start.elapsed());
    Ok(StatusCode::CREATED)
}