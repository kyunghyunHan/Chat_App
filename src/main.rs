use futures::StreamExt;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
type Users = Arc<RwLock<HashMap<usize, mpsc::UnboundedSender<Result<Message, warp::Error>>>>>;
#[tokio::main]
async fn main() {
    let address = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8000".to_string());
    let socket_address: SocketAddr = address.parse().expect("valid socket Address");
    println!("address{:?}", address);
    println!("socket_address{:?}", socket_address);

    let users = Users::default();
    let users = warp::any().map(move || users.clone());

    let opt = warp::path::param::<String>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<String>,), std::convert::Infallible>((None,)) });
}
