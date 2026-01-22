use serde::{Deserialize, Serialize};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{js_sys, MessageEvent, WebSocket};
use yew::Callback;

use sdr_db::spectrum_types::SpectrumFrame;

#[derive(Debug)]
pub struct SpectrumClient {
    ws: WebSocket,
}
impl SpectrumClient {
    pub fn connect(url: &str, on_frame: impl Fn(SpectrumFrame) + 'static) -> Self {
        let ws = WebSocket::new(url).unwrap();
        let onmessage = Closure::wrap(Box::new(move |message_event: MessageEvent| {
            if let Some(text) = message_event.data().as_string() {
                if let Ok(frame) = serde_json::from_str::<SpectrumFrame>(&text) {
                    on_frame(frame);
                }
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        SpectrumClient { ws }
    }
}
/*
pub fn connect_spectrum(url: &str, on_data: Callback<SpectrumFrame>) {
    let ws = WebSocket::new(url).unwrap();
  let onmessage = Closure::wrap(Box::new(move |message_event: MessageEvent| {
          if let Ok(text) = message_event.data().dyn_into::<JsString>() {
              let frame: SpectrumFrame = serde_json::from_str(&text.as_string().unwrap()).unwrap();
              on_data.emit(frame);
          }
      }) as Box<dyn FnMut(MessageEvent)>);

      ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
      onmessage.forget();
}
*/
