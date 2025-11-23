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
