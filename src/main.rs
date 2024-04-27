#![feature(lazy_cell)]

use axum::{Router, ServiceExt};
use events::daily_challenge::run_daily_challenge;
use serenity::http::Http;
use serenity::prelude::Context;
use serenity::Client;
use shuttle_runtime::SecretStore;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

pub mod config;
pub mod events;
pub mod general_commands;
pub mod slash_commands;
use config::setup::setup;

pub struct CustomService {
    discord_bot: Client,
    router: Router,
}

#[shuttle_runtime::async_trait]
impl shuttle_runtime::Service for CustomService {
    async fn bind(mut self, addr: SocketAddr) -> Result<(), shuttle_runtime::Error> {
        let router = self.router.into_service();

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        let serve_router = async move {
            axum::serve(listener, router.into_make_service())
                .await
                .unwrap();
        };

        tokio::select! {
            _ = self.discord_bot.start_autosharded() => {},
            _ = serve_router => {},
        };

        Ok(())
    }
}

fn build_router(ctx: Arc<Http>) -> Router {
    Router::new()
        .route("/daily_challenge", axum::routing::post(run_daily_challenge))
        .with_state(ctx)
}

#[shuttle_runtime::main]
async fn init(
    #[shuttle_runtime::Secrets] secret_store: SecretStore,
) -> Result<CustomService, shuttle_runtime::Error> {
    let Ok(_) = color_eyre::install() else {
        panic!("Failed to install color_eyre");
    };
    let public_folder = PathBuf::from("static");

    let discord_bot = setup(secret_store, public_folder).await?;
    let router = build_router(discord_bot.http.clone());

    Ok(CustomService {
        discord_bot,
        router,
    })
}
