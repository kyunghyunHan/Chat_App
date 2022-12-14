use futures::StreamExt;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use warp::Filter;
static NEXT_USERID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
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

    let hello = warp::path("hello")
        .and(opt)
        .and(warp::path::end())
        .map(|name: Option<String>| {
            format!("Hello,{}!", name.unwrap_or_else(|| "world".to_string()))
        });

    let chat = warp::path("ws")
        .and(warp::ws())
        .and(users)
        .map(|ws: warp::ws::Ws, users| ws.on_upgrade(move |socket| connect(socket, users)));

    let files = warp::fs::dir("./static");

    let res_404 = warp::any().map(|| {
        warp::http::Response::builder()
            .status(warp::http::StatusCode::NOT_FOUND)
            .body(fs::read_to_string("./static/404.html").expect("404"))
    });

    let routes = chat.or(hello).or(files).or(res_404);

    let server = warp::serve(routes).try_bind(socket_address);

    println!("Running server{}", address);

    server.await
}

async fn connect(ws: WebSocket, users: Users) {
    let my_id = NEXT_USERID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    println!("welcome{}", my_id);

    let (user_tx, mut user_rx) = ws.split();
    println!("{:?}", user_tx);
    let (tx, rx) = mpsc::unbounded_channel();
    println!("{:?}", tx);
    let rx = UnboundedReceiverStream::new(rx);

    tokio::spawn(rx.forward(user_tx));
    users.write().await.insert(my_id, tx);

    while let Some(result) = user_rx.next().await {
        broadcast_msg(result.expect("filed message"), &users).await;
    }

    disconnect(my_id, &users).await;
}
async fn broadcast_msg(msg: Message, users: &Users) {
    println!("{:?}", msg);
    if let Ok(_) = msg.to_str() {
        for (&_uid, tx) in users.read().await.iter() {
            tx.send(Ok(msg.clone())).expect("Failed")
        }
    }
}

async fn disconnect(my_id: usize, users: &Users) {
    println!("Good bye,{}", my_id);
    users.write().await.remove(&my_id);
}
