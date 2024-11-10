// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::routing::get_service;
use axum::Router;
use pnet::datalink::interfaces;
use std::net::SocketAddr;
use structopt::StructOpt;
use tower_http::services::ServeDir;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long)]
    html: Option<String>,

    #[structopt(short, long, default_value = "self")]
    host: String,

    #[structopt(long, default_value = "8080")]
    port: u16,
}

/// Run an `axum::Router` on incoming requests from a socket.
pub async fn run_router_on_socket(router: Router) -> Result<(), String> {
    let options = Options::from_args();
    let port = options.port;
    let addr = if options.host == "self" {
        let all_interfaces = interfaces();
        let external_addr = all_interfaces
            .iter()
            .find(|e| e.is_up() && !e.is_loopback() && !e.ips.is_empty())
            .and_then(|interface| interface.ips.first())
            .and_then(|item| Some(SocketAddr::from((item.ip(), port))));
        external_addr
    } else {
        // e.g. --host "foo:8080" (this ignores --port)
        options.host.parse().ok()
    };

    let router = if let Some(path) = options.html {
        router.fallback_service(get_service(ServeDir::new(path)))
    } else {
        router
    };

    if let Some(addr) = addr {
        println!("Begin running router on socket {}", addr);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("{e:?}"))?;
        axum::serve(listener, router.into_make_service())
            .await
            .map_err(|e| format!("{e:?}"))?;
        println!("Done running router on socket");
        Ok(())
    } else {
        Err("invalid address".to_string())
    }
}
