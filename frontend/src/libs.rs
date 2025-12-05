use futures::{SinkExt, channel::mpsc::UnboundedSender};
use gloo_net::websocket::Message;
use structs::MessageText;
use sycamore::{prelude::ReadSignal, web::console_error};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Clipboard, window};

pub async fn copy_to_clipboard(text: &str) -> Result<(), JsValue> {
    let nav = window().ok_or("no window")?.navigator();
    // La API Clipboard puede no estar disponible en contextos inseguros
    let clipboard: Clipboard = nav.clipboard();
    JsFuture::from(clipboard.write_text(text)).await?;
    Ok(())
}

pub async fn send_message(
    send: ReadSignal<Option<UnboundedSender<Message>>>,
    message: MessageText,
) {
    match send.get_clone() {
        Some(mut sender) => {
            if let Err(e) = sender
                .send(Message::Text(
                    serde_json::to_string(&message).unwrap_or_default(),
                ))
                .await
            {
                console_error!("{e}");
            }
        }
        None => {
            console_error!("Error: No Socket available");
        }
    }
}
