use axum::response::IntoResponse;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Mutex, Arc};
use axum::extract::{State, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::handler::{Handler, HandlerWithoutStateExt};
use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast::{Sender, Receiver};
use tokio::task;
use crate::{arc, arc_mutex, mutex, string};
use crate::structs::{Role, User, MessageText};
use std::env;
use dotenv::dotenv;

// #[derive(Debug, Clone)]
// struct Room {
//     id: String,
//     users: Arc<Mutex<HashSet<User>>>,
//     tx: Sender<String>
// }
// impl PartialEq for Room {
//     fn eq(&self, other: &Self) -> bool {
//         self.id == other.id
//     }
// }
//
// #[derive(Debug, Clone)]
// struct AppState {
//     rooms: Arc<Mutex<HashSet<Room>>,
// }

#[derive(Debug, Clone)]
struct AppState {
    users: Arc<Mutex<HashSet<User>>>,
    tx: Sender<String>
}
pub async fn run() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();
    let users:HashSet<User> = HashSet::new();
    let (tx, rx) = broadcast::channel::<String>(20);
    // let state = Arc::new(tx);
    let state = AppState{ users: Arc::new(Mutex::new(users)), tx };


    let app = Router::new().route("/ws", get(ws_handler)).with_state(state);


    let addr: SocketAddr = env::var(string!("HOST")).expect("HOST not set").parse().unwrap();
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server in {}", addr);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move {
        handle_socket(socket, state).await
    })
}
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    println!("Client connected");
    let tx_clone = state.tx.clone();
    let mut rx = state.tx.subscribe();
    let this_user = arc_mutex!(None::<User>);
    let this_other = this_user.clone();
    let (mut send,mut recv) = socket.split();
    let arc_users = state.users.clone();
    let mut send_task = task::spawn(async move {
        {
            let lock = state.users.lock().unwrap().iter().cloned().collect::<Vec<User>>();
            if send.send(Message::Text(serde_json::to_string(&lock).unwrap().into())).await.is_err() {
                println!("Could not send user lock");
            }
        }
        while let Ok(msg) = rx.recv().await {
            println!("Received message: {}", msg);
            match serde_json::from_str::<Vec<User>>(&msg) {
                Ok(message) => {
                    println!("Message Text: {:#?}", message);
                    // state.users.lock().unwrap().insert(message.user.clone());
                    // let mut lock = this_other.lock().unwrap();
                    // *lock = Some(message.user);
                    // println!("Users: {:#?}",state.users)
                },
                Err(e) => println!("Error parsing message: {:#?}", e),
            }
            let lock = state.users.lock().unwrap().iter().cloned().collect::<Vec<User>>();
            if send.send(Message::Text(serde_json::to_string(&lock).unwrap().into())).await.is_err() {
                println!("Disconnected 86");
                break;
            }
        }
        if let Some(user) = this_other.lock().unwrap().as_ref().cloned() {
            println!("Removing user {:#?}",user);
            state.users.lock().unwrap().remove(&user);
        } else {
            println!("No user found {:#?}",this_other);
        }
        println!("Disconnected 90");
    });


    let mut recv_task = task::spawn(async move {
        while let Some(Ok(Message::Text(msg))) = recv.next().await {
            println!("Received {}",msg);
            match serde_json::from_str::<MessageText>(msg.as_str()) {
                Ok(message) => {
                    // match state.rooms.iter().find(|room|room.id.eq(&message.room)){
                    //     Some(room) => {},
                    //     None => {}
                    // }

                    arc_users.lock().unwrap().insert(message.user.clone());
                    this_user.lock().unwrap().replace(message.user);
                }
                Err(e) => println!("Error parsing message: {:#?}", e),
            }
            let lock = arc_users.lock().unwrap().iter().cloned().collect::<Vec<User>>();
            tx_clone.send(serde_json::to_string(&lock).unwrap()).unwrap();
        }
        println!("To remove user {:#?}",this_user);
        if let Some(user) = this_user.lock().unwrap().as_ref() {
            println!("Removing user {:#?}",user);
            let mut users_lock = arc_users.lock().unwrap();
            users_lock.remove(user);
            tx_clone.send(serde_json::to_string(&users_lock.iter().cloned().collect::<Vec<User>>()).unwrap()).unwrap();
        }

        println!("Disconnected 99");
    });

    tokio::select! {
        a = (&mut send_task) => recv_task.abort(),
        b = (&mut recv_task) => send_task.abort(),
    }

}