use crate::structs::{MessageBack, MessageText, Role, User};
use crate::{arc, arc_mutex, mutex, string};
use axum::Router;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::handler::{Handler, HandlerWithoutStateExt};
use axum::response::IntoResponse;
use axum::routing::get;
use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, Mutex, MutexGuard};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task;
use uuid::Uuid;

#[derive(Debug, Clone)]
struct Room {
    id: String,
    users: Arc<Mutex<HashSet<User>>>,
    tx: Sender<String>,
}
impl PartialEq for Room {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Clone)]
struct AppState {
    rooms: Arc<Mutex<Vec<Room>>>,
}

// impl AppState {
//     fn get_room(&self, id: &str) -> Option<&Room> {
//         self.rooms.lock().await.iter().find(|room| room.id == id)
//     }
// }

// #[derive(Debug, Clone)]
// struct AppState {
//     users: Arc<Mutex<HashSet<User>>>,
//     tx: Sender<String>
// }
pub async fn run() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    dotenv().ok();
    let users: HashSet<User> = HashSet::new();
    let (tx, rx) = broadcast::channel::<String>(20);
    // let state = Arc::new(tx);
    let state = AppState {
        rooms: arc_mutex!(vec![]),
    };

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let addr: SocketAddr = env::var(string!("HOST"))
        .expect("HOST not set")
        .parse()
        .unwrap();
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server in {}", addr);

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| async move { handle_socket(socket, state).await })
}
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    println!("Client connected");
    let this_user: Arc<Mutex<Option<User>>> = arc_mutex!(None);
    let this_other = this_user.clone();
    let (mut send, mut recv) = socket.split();
    // let arc_users = state.users.clone();
    let arc_rooms = state.rooms.clone();
    let mut send_task = task::spawn(async move {
        'outer: loop {
            let user_lock;
            {
                user_lock = this_other.lock().await.clone();
            }
            if let Some(user) = user_lock {
                let rooms_lock;
                {
                    rooms_lock = arc_rooms
                        .lock()
                        .await
                        .iter()
                        .find(|room| room.id.eq(user.room()))
                        .cloned();
                }
                if let Some(room) = rooms_lock {
                    let mut rx = room.tx.subscribe();
                    while let Ok(msg) = rx.recv().await {
                        dbg!(&msg);
                        // match serde_json::from_str::<User>(&msg) {
                        //     Ok(message) => {
                        //         dbg!(&message);
                        //
                        //         // lock.replace(message.)
                        //         // state.users.lock().await.insert(message.user.clone());
                        //         // let mut lock = this_other.lock().await;
                        //         // *lock = Some(message.user);
                        //         // println!("Users: {:#?}",state.users)
                        //     }
                        //     Err(e) => { dbg!(&e); },
                        // }
                        let user_lock;
                        {
                            user_lock = this_other.lock().await.clone();
                        }
                        if let Some(user_lock) = user_lock{
                            let room_lock;
                            {
                                room_lock = arc_rooms.lock().await.iter().cloned().find(|room|{
                                    room.id.eq(user_lock.room())
                                });
                            }
                            match room_lock {
                                Some(room) => {
                                    let users;
                                    {
                                        users = room.users.lock().await.iter().cloned().collect::<Vec<User>>();
                                    }
                                    if let Err(e) = send.send(Message::Text(serde_json::to_string(&MessageBack{users, room: room.id.to_owned()}).unwrap().into())).await{
                                        println!("Disconnected 86");
                                        break;
                                    }
                                },
                                None => {}
                            }

                        }
                        // let lock = state.users.lock().await.iter().cloned().collect::<Vec<User>>();
                        // if send.send(Message::Text(serde_json::to_string(&lock).unwrap().into())).await.is_err() {
                        //     println!("Disconnected 86");
                        //     break;
                        // }
                    }
                    break 'outer;

                    // println!("Removing user {:#?}",user);
                    // state.users.lock().await.remove(&user);

                    println!("Disconnected 90");
                }
            }
        }
        // {
        //     let lock = state.users.lock().await.iter().cloned().collect::<Vec<User>>();
        //     if send.send(Message::Text(serde_json::to_string(&lock).unwrap().into())).await.is_err() {
        //         println!("Could not send user lock");
        //     }
        // }
        println!("ending")
    });

    let mut recv_task = task::spawn(async move {
        while let Some(Ok(Message::Text(msg))) = recv.next().await {
            match serde_json::from_str::<MessageText>(msg.as_str()) {
                Ok(message) => {
                    let mut rooms_lock = state.rooms.lock().await;
                    let mut new_user = message.user;
                    if new_user.role() == Role::Master {
                        new_user.set_room(Uuid::new_v4().to_string());
                    }
                    match rooms_lock
                        .iter()
                        .cloned()
                        .find(|room| room.id.eq(&new_user.room()))
                    {
                        Some(room) => {
                            room.users.lock().await.replace(new_user.to_owned());
                        }
                        None => {
                            let new_room = Room {
                                users: arc_mutex!(HashSet::from([new_user.to_owned()])),
                                tx: broadcast::channel::<String>(20).0,
                                id: new_user.room().to_owned(),
                            };
                            rooms_lock.push(new_room);
                        }
                    }


                    this_user.lock().await.replace(new_user.to_owned());
                }
                Err(e) => { dbg!(&e); },
            }
            if let Some(user) = this_user.lock().await.as_ref() {
                if let Some(room) = state
                    .rooms
                    .lock()
                    .await
                    .iter()
                    .cloned()
                    .find(|room| room.id.eq(user.room()))
                {
                    room.tx
                        .send(
                            serde_json::to_string(
                                user,
                            )
                            .unwrap(),
                        )
                        .unwrap();
                }
            }
            // let lock = arc_users.lock().await.iter().cloned().collect::<Vec<User>>();
            // tx_clone.send(serde_json::to_string(&lock).unwrap()).unwrap();
        }
        dbg!(&this_user);
        if let Some(user) = this_user.lock().await.as_ref() {
            if let Some(room) = state
                .rooms
                .lock()
                .await
                .iter()
                .cloned()
                .find(|room| room.id.eq(user.room()))
            {
                let mut users_lock = room.users.lock().await;
                users_lock.remove(user);
                room.tx
                    .send(
                        serde_json::to_string(&users_lock.iter().cloned().collect::<Vec<User>>())
                            .unwrap(),
                    )
                    .unwrap();
            }
        }
        // if let Some(user) = this_user.lock().await.as_ref() {
        //     println!("Removing user {:#?}",user);
        //     let mut users_lock = arc_users.lock().await;
        //     users_lock.remove(user);
        //     tx_clone.send(serde_json::to_string(&users_lock.iter().cloned().collect::<Vec<User>>()).unwrap()).unwrap();
        // }

        println!("Disconnected 99");
    });

    tokio::select! {
        a = (&mut send_task) => recv_task.abort(),
        b = (&mut recv_task) => send_task.abort(),
    }

    println!("end of handle 187");
}
