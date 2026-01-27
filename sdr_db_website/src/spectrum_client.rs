use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{console, MessageEvent, WebSocket};

use sdr_db::spectrum_types::SpectrumFrame;

pub struct SpectrumClient {
    ws: WebSocket,
    // Store closure to prevent it from being dropped
    _onmessage: Closure<dyn FnMut(MessageEvent)>,
}

impl SpectrumClient {
    pub fn connect(url: &str, on_frame: impl Fn(SpectrumFrame) + 'static) -> Self {
        let ws = WebSocket::new(url).expect("Failed to create WebSocket");

        let onmessage = Closure::wrap(Box::new(move |message_event: MessageEvent| {
            if let Some(text) = message_event.data().as_string() {
                match serde_json::from_str::<SpectrumFrame>(&text) {
                    Ok(frame) => {
                        on_frame(frame);
                    }
                    Err(e) => {
                        console::error_1(&format!("[spectrum] Parse error: {}", e).into());
                    }
                }
            } else {
                console::warn_1(&"[spectrum] Non-text message received".into());
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));

        SpectrumClient {
            ws,
            _onmessage: onmessage,
        }
    }

    pub fn drop(self) {
        self.ws.close().unwrap();
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
