mod commands;
mod data;
mod tasks;

use std::env::var;
use std::sync::Arc;
use std::time::Duration;

use commands::tags::tag_utils::TagDb;
use data::database::create_users_table;
use dotenv::dotenv;
use mongodb::Client;
use mongodb::options::ClientOptions;
use poise::serenity_prelude as serenity;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tasks::update_uptime::uptime_updater;
use tokio::sync::OnceCell;
use types::{Context, Error};

mod types {
	pub type Error = Box<dyn std::error::Error + Send + Sync>;
	pub type Context<'a> = poise::Context<'a, super::Data, Error>;
}

pub struct Data {
	pub tag_db: Arc<TagDb>,
}

static MONGO_CLIENT: OnceCell<Client> = OnceCell::const_new();
static API_KEY: OnceCell<String> = OnceCell::const_new();
static ERROR_COLOR: OnceCell<u32> = OnceCell::const_new();
static DB_POOL: OnceCell<Pool<SqliteConnectionManager>> = OnceCell::const_new();

async fn init_global_data() {
	API_KEY
		.set(
			var("API_KEY")
				.expect_error("Missing `API_KEY` env var, please include this in your .env file"),
		)
		.expect_error("API_KEY can only be initialized once");

	let mongo_url = var("MONGO_URL")
		.expect_error("Missing `MONGO_URL` env var, please include this in your .env file");
	let options = ClientOptions::parse(mongo_url)
		.await
		.expect_error("Could not create mongo client options");
	let client = Client::with_options(options).expect_error("Could not create mongo client");

	MONGO_CLIENT
		.set(client)
		.expect_error("MONGO_CLIENT can only be initialized once");

	ERROR_COLOR
		.set(0x770505)
		.expect_error("ERROR_COLOR can only be initialized once");

	let manager = SqliteConnectionManager::file("src/data/tags.db");
	let pool = Pool::new(manager).expect_error("Failed to create connection pool");

	DB_POOL
		.set(pool)
		.expect_error("DB_POOL can only be initialized once");
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
	match error {
		| poise::FrameworkError::Setup { error, .. } => {
			panic!("\x1b[31;1m[ERROR] Failed to start bot:\x1b[0m {:?}", error)
		},
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

pub trait ExpectError<T> {
	fn expect_error(
		self,
		msg: &str,
	) -> T;
}

impl<T, E: std::fmt::Debug> ExpectError<T> for Result<T, E> {
	fn expect_error(
		self,
		msg: &str,
	) -> T {
		self.expect(&format!("\x1b[31;1m[ERROR] {}\x1b[0m", msg))
	}
}

#[tokio::main]
async fn main() {
	dotenv().ok();

	let options = poise::FrameworkOptions {
		commands: commands::get_all_commands(),
		prefix_options: poise::PrefixFrameworkOptions {
			prefix: Some("cow ".into()),
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
		skip_checks_for_owners: false,
		event_handler: |_ctx, event, _framework, _data| {
			Box::pin(async move {
				println!("[EVENT HANDLER] {:?}", event.snake_case_name());
				Ok(())
			})
		},
		..Default::default()
	};

	init_global_data().await;
	create_users_table().expect_error("Failed to create database \'users\'");

	let framework = poise::Framework::builder()
		.setup(move |ctx, _ready, framework| {
			Box::pin(async move {
				println!("Logged in as {}", _ready.user.name);
				poise::builtins::register_globally(ctx, &framework.options().commands).await?;
				Ok(Data {
					tag_db: Arc::new(TagDb),
				})
			})
		})
		.options(options)
		.build();

	tokio::task::spawn_blocking(move || {
		if let Err(err) = tokio::runtime::Handle::current().block_on(uptime_updater(
			&API_KEY.get().unwrap(),
			MONGO_CLIENT
				.get()
				.unwrap()
				.database("Players")
				.collection("Uptime"),
		)) {
			eprintln!(
				"\x1b[31;1m[ERROR] Error in uptime tracker:\x1b[0m\n\n{:?}",
				err
			);
		}
	});

	let token = var("BOT_TOKEN").expect_error(
		"\x1b[31;1m[ERROR] Missing `BOT_TOKEN` env var, please include this in your .env file",
	);
	let intents =
		serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

	let client = serenity::ClientBuilder::new(token, intents)
		.framework(framework)
		.await;

	client.unwrap().start().await.unwrap();
}
