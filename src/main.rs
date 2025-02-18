#![warn(clippy::str_to_string)]

#[path = "commands/color.rs"]
mod color;
#[path = "commands/get_linked_account.rs"]
mod get_linked_account;
#[path = "commands/uptime.rs"]
mod uptime;

mod commands;

#[path = "data/database.rs"]
mod database;
#[path = "data/update_uptime.rs"]
mod update_uptime;

use std::env::var;
use std::sync::Arc;
use std::time::Duration;

use database::{create_uptime_table, create_users_table};
use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use types::{Context, Error};

mod types {
	pub type Error = Box<dyn std::error::Error + Send + Sync>;
	pub type Context<'a> = poise::Context<'a, super::Data, Error>;
}

// Custom user data passed to all command functions
pub struct Data {
	api_key:        String,
	uptime_db_pool: Pool<SqliteConnectionManager>,
	error_color:    u32,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
	match error {
		| poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
		| poise::FrameworkError::Command { error, ctx, .. } => {
			println!(
				"\x1b[31;1m[ERROR] in command '{}':\x1b[0m {:?}",
				ctx.command().name,
				error
			);
		},
		| error => {
			if let Err(e) = poise::builtins::on_error(error).await {
				println!("\x1b[31;1m[ERROR] while handling error:\x1b[0m {}", e)
			}
		},
	}
}

#[tokio::main]
async fn main() {
	dotenv().ok();
	env_logger::init();

	let api_key = var("API_KEY").expect("\x1b[31;1m[ERROR] Missing `API_KEY` env var, please include this in the environment variables or features may not work\x1b[0m");

	let options = poise::FrameworkOptions {
		commands: vec![
			get_linked_account::get_linked_account(),
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

	create_users_table().expect("\x1b[31;1m[ERROR] Failed to create database 'users'\x1b[0m\n\n");
	create_uptime_table().expect("\x1b[31;1m[ERROR] Failed to create database 'uptime'\x1b[0m\n\n");

	// create uptime connection pool
	let manager = SqliteConnectionManager::file("src/data/uptime.db")
		.with_flags(rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE);
	let uptime_db_pool = Pool::new(manager).expect("Failed to create database pool");

	let clone_api_key = api_key.clone();
	let framework = poise::Framework::builder()
		.setup(move |ctx, _ready, framework| {
			Box::pin(async move {
				println!("Logged in as {}", _ready.user.name);
				poise::builtins::register_globally(ctx, &framework.options().commands).await?;
				Ok(Data {
					api_key: clone_api_key,
					uptime_db_pool,
					error_color: 0x383838,
				})
			})
		})
		.options(options)
		.build();

	tokio::task::spawn_blocking(move || {
		if let Err(err) =
			tokio::runtime::Handle::current().block_on(update_uptime::update_uptime(&api_key))
		{
			eprintln!(
				"\x1b[31;1m[ERROR] Error in uptime tracker:\x1b[0m\n\n{:?}",
				err
			);
		}
	});

	let token = var("BOT_TOKEN").expect(
		"\x1b[31;1m[ERROR] Missing `BOT_TOKEN` env var, please include this in the environment variables\x1b[0m",
	);
	let intents =
		serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

	let client = serenity::ClientBuilder::new(token, intents)
		.framework(framework)
		.await;

	client.unwrap().start().await.unwrap();
}
