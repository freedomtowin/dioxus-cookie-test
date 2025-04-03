#![allow(non_snake_case, unused)]
use dioxus::prelude::*;
use serde::{Serialize, Deserialize};
use tower_cookies::{Cookies, CookieManagerLayer};
use web_sys::{window, Document, HtmlDocument}; // For client-side cookie access
use js_sys::Date;
use js_sys::wasm_bindgen::JsCast;

#[cfg(feature = "server")]
mod backend;

fn main() {
    // Initialize the logger
    dioxus::logger::initialize_default();

    #[cfg(feature = "web")]
    // Hydrate the application on the client
    LaunchBuilder::web().launch(app);

    #[cfg(feature = "server")]
    {
        tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(backend::launch());
    }
}

fn app() -> Element {
    let mut refresh_key = use_signal(|| 0); // To force resource refresh
    let auth_status = use_resource(move || async move {
        // Read cookies on the client side
        // Depend on refresh_key to re-run when it changes
        let _ = refresh_key.read(); // Access signal to create dependency
        match get_cookie("auth_token") {
            Ok(Some(token)) => AuthStatus::LoggedIn(token),
            Ok(None) => AuthStatus::NotLoggedIn,
            Err(e) => AuthStatus::Error(e),
        }
    });

    rsx! {
        div {
            button {
                onclick: move |_| {
                    // Set the cookie when clicked
                    if let Err(e) = set_cookie("auth_token", "example_id_token_from_cognito", 1) {
                        println!("Failed to set cookie: {}", e);
                    }
                    // Increment refresh_key to trigger resource re-run
                    let key = *refresh_key.read(); 
                    refresh_key.set(key + 1);
                },
                "Set Cookie"
            }
            div {
                match &*auth_status.read_unchecked() {
                    Some(AuthStatus::LoggedIn(token)) => format!("Logged in with token: {}", token),
                    Some(AuthStatus::NotLoggedIn) => "Not logged in".to_string(),
                    Some(AuthStatus::Error(msg)) => "Error: {msg}".to_string(),
                    None => "Loading...".to_string(),
                }
            }
        }
    }
}

// Client-side utility to get a cookie by name
fn get_cookie(name: &str) -> Result<Option<String>, String> {
    let window = window().ok_or("No window available")?;
    let document = window.document().ok_or("No document available")?;
    let html_document = document.dyn_into::<HtmlDocument>().map_err(|_| "Failed to cast to HtmlDocument")?;
    
    if let Some(cookies) = html_document.cookie().ok() {
        for cookie in cookies.split(';') {
            let cookie = cookie.trim();
            if cookie.starts_with(&format!("{}=", name)) {
                let value = cookie[name.len() + 1..].to_string();
                return Ok(Some(value));
            }
        }
        Ok(None)
    } else {
        Err("Failed to access cookies".to_string())
    }
}

// Optional: Client-side utility to set a cookie
fn set_cookie(name: &str, value: &str, days: u32) -> Result<(), String> {
    let window = window().ok_or("No window available")?;
    let document = window.document().ok_or("No document available")?;
    let html_document = document.dyn_into::<HtmlDocument>().map_err(|_| "Failed to cast to HtmlDocument")?;
    
    let expires = Date::new_0();
    expires.set_date(expires.get_date() + days);
    let cookie_str = format!(
        "{}={}; expires={}; path=/",
        name,
        value,
        expires.to_utc_string().as_string().unwrap_or_default()
    );
    html_document.set_cookie(&cookie_str).map_err(|_| "Failed to set cookie")?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AuthStatus {
    LoggedIn(String),    // Contains the token
    NotLoggedIn,         // No token present
    Error(String),       // Error message
}
