

use crate::ServeConfigBuilder;
use dioxus::prelude::DioxusRouterExt;
use tokio::net::TcpListener;
use dioxus::prelude::*;
use tower_cookies::{Cookies, CookieManagerLayer};

pub async fn launch() {

    use axum::{Router};
    // register server functions
    let router = Router::new()
    
    .serve_dioxus_application(
        ServeConfigBuilder::new(),
        super::app // or whatever your main component is
    )
    .into_make_service();

    // start server
    let socket_addr = dioxus_cli_config::fullstack_address_or_localhost();
    let listener = TcpListener::bind(socket_addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

