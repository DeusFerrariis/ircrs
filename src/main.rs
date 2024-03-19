#![feature(async_closure)]

#[macro_use] extern crate rocket;

use rocket_ws::{WebSocket, Stream};
use std::sync::atomic::AtomicUsize;
use std::time::{ Instant, Duration };
use tokio::time::sleep;
use crate::rocket::futures::StreamExt;
use crate::rocket::futures::SinkExt;
use rocket::State;
use std::cmp::Ordering;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::{
    FutureExt,
    pin_mut,
    select,
};

#[get("/")]
async fn index() -> &'static str {
    "Hello, world!"
}

struct Messages {
    count: usize,
    messages: Vec<String>,
}

type MessagesState = Arc<Mutex<Messages>>;

#[get("/echo")]
async fn echo_stream(msgs: &State<MessagesState>, ws: WebSocket) -> rocket_ws::Channel<'_> {
    ws.channel(move |mut stream| Box::pin(async move {
        let mut unsynced_messages: Vec<String> = vec![];
        let mut last_read = 0;
        let mut alias: String = "".to_string();

        loop {
            let alias_req = format!("\nWhat is your alias?");
            let _ = stream.send(alias_req.into()).await;
            if let Some(msg) = stream.next().await {
                if let Ok(rocket_ws::Message::Text(a)) = msg {
                    alias = a;
                    break;
                }
            }
        }

        loop {
            let t1 = msgs.lock().fuse();
            let t2 = stream.next().fuse();

            pin_mut!(t1, t2);

            select! {
                mut lock = t1 => {
                    if lock.messages.len() > last_read {
                        for m in &mut lock.messages[last_read..] {
                            let msg = format!("{}", m);
                            let _ = stream.send(msg.into()).await;
                        }

                        last_read = lock.messages.len();
                    }

                    if unsynced_messages.len() > 0 {
                        lock.messages.append(&mut unsynced_messages);
                    }
                    continue;
                },
                maybe_message = t2 => {
                    if let Some(message) = maybe_message {
                        if let Ok(rocket_ws::Message::Text(msg)) = message {
                            unsynced_messages.push(format!("[{}]: {}", alias, msg));
                        }
                    }
                    continue;
                }
            }
        }

        Ok(())
    }))
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(Arc::new(Mutex::new(Messages { messages: vec![], count: 0 })))
        .mount("/", routes![index, echo_stream])
}
