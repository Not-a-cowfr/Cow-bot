#![warn(clippy::str_to_string)]

#[path = "commands/uptime.rs"]
mod uptime;

#[path = "commands/color.rs"]
mod color;

mod commands;
#[path = "data/database.rs"]
mod database;

use std::env::var;
use std::sync::Arc;
use std::time::Duration;

use database::create_users_table;
use dotenv::dotenv;
use poise::serenity_prelude as serenity;

mod types {
	pub type Error = Box<dyn std::error::Error + Send + Sync>;
	pub type Context<'a> = poise::Context<'a, super::Data, Error>;
}

use types::{Context, Error};

// Custom user data passed to all command functions
pub struct Data {
	api_key: String,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
	match error {
		| poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
		| poise::FrameworkError::Command { error, ctx, .. } => {
			println!("[ERROR] in command '{}': {:?}", ctx.command().name, error);
		},
		| error => {
			if let Err(e) = poise::builtins::on_error(error).await {
				println!("[ERROR] while handling error: {}", e)
			}
		},
	}
}

#[tokio::main]
async fn main() {
	dotenv().ok();
	env_logger::init();

	let api_key = var("API_KEY").expect("[ERROR] Missing `API_KEY` env var, please include this in the environment variables or features may not work");

	let options = poise::FrameworkOptions {
		commands: vec![
			uptime::get_linked_account(),
			uptime::uptime(),
			color::color(),
		],
		prefix_options: poise::PrefixFrameworkOptions {
			prefix: Some(";".into()),
			edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
				Duration::from_secs(3600),
			))),
			..Default::default()
		},
		on_error: |error| Box::pin(on_error(error)),
		pre_command: |ctx| {
			Box::pin(async move {
				println!("[COMMAND] started {}", ctx.command().qualified_name);
			})
		},
		post_command: |ctx| {
			Box::pin(async move {
				println!("[COMMAND] completed {}", ctx.command().qualified_name);
			})
		},
		command_check: Some(|ctx| {
			Box::pin(async move {
				if ctx.author().id == 123456789 {
					return Ok(false);
				}
				Ok(true)
			})
		}),
		skip_checks_for_owners: false,
		event_handler: |_ctx, event, _framework, _data| {
			Box::pin(async move {
				println!("[EVENT HANDLER] {:?}", event.snake_case_name());
				Ok(())
			})
		},
		..Default::default()
	};

	let framework = poise::Framework::builder()
		.setup(move |ctx, _ready, framework| {
			let api_key = api_key.clone();
			Box::pin(async move {
				println!("Logged in as {}", _ready.user.name);
				poise::builtins::register_globally(ctx, &framework.options().commands).await?;
				Ok(Data { api_key })
			})
		})
		.options(options)
		.build();

	create_users_table().expect("[ERROR] Failed to create database 'users'");

	let token = var("BOT_TOKEN").expect(
		"[ERROR] Missing `BOT_TOKEN` env var, please include this in the environment variables",
	);
	let intents =
		serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

	let client = serenity::ClientBuilder::new(token, intents)
		.framework(framework)
		.await;

	client.unwrap().start().await.unwrap()
}
