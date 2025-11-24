use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Geolocation, Position, PositionError};
use yew::hook;
use yew_hooks::prelude::*;

#[hook]
pub fn use_geolocation() -> UseGeolocationState {
    use_geolocation_with_options(UseGeolocationOptions::default())
}

pub fn get_geolocation() -> Option<Geolocation> {
    web_sys::window()?.navigator().geolocation().ok()
}

pub fn coords_to_gridsquare(latitude: f64, longitude: f64) -> String {
    sdr_db::to_maidenhead(latitude, longitude).unwrap()
}

pub fn position_to_gridsquare(position: &Position) -> String {
    let coords = position.coords();
    let lat = coords.latitude();
    let lon = coords.longitude();
    coords_to_gridsquare(lat, lon)
}

pub fn request_gridsquare<F>(geolocation: &Geolocation, callback: F)
where
    F: Fn(String) + 'static,
{
    let success_callback = Closure::once(move |position: Position| {
        let gridsquare = position_to_gridsquare(&position);
        callback(gridsquare);
    });

    let error_callback = Closure::once(move |error: PositionError| {
        web_sys::console::log_1(&format!("Geolocation error: {:?}", error.message()).into());
    });

    let _ = geolocation.get_current_position_with_error_callback(
        success_callback.as_ref().unchecked_ref(),
        Some(error_callback.as_ref().unchecked_ref()),
    );

    success_callback.forget();
    error_callback.forget();
}
