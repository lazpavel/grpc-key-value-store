use lambda_extension::tracing::info;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, convert::Infallible};
use warp::Filter;
use kvstore::key_value_store_client::KeyValueStoreClient;
use tonic::transport::Channel;

use crate::{handlers, kvstore};

pub fn cache_routes(
    client: Arc<Mutex<KeyValueStoreClient<Channel>>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    info!("setting up routes");
    get_item(client.clone())
        .or(set_item(client.clone()))
}

fn set_item(
    client: Arc<Mutex<KeyValueStoreClient<Channel>>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("set")
        .and(warp::post())
        .and(warp::body::content_length_limit(5 * 1024 * 1024)) // 5 MB limit for request
        .and(json_body())
        .and(with_client(client))
        .and_then(handlers::set_handler)
}

fn get_item(
    client: Arc<Mutex<KeyValueStoreClient<Channel>>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("get" / String)
        .and(warp::get())
        .and(with_client(client))
        .and_then(handlers::get_handler)
}

fn json_body() -> impl Filter<Extract = (HashMap<String, String>,), Error = warp::Rejection> + Clone
{
    warp::body::json()
}

fn with_client(
    client: Arc<Mutex<KeyValueStoreClient<Channel>>>,
) -> impl Filter<Extract = (Arc<Mutex<KeyValueStoreClient<Channel>>>,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}
