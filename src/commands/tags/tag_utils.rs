use std::fmt;

use poise::CreateReply;
use tokio::task;

use crate::{types::Context, Data, ExpectError, DB_POOL};

use crate::commands::utils::create_error_embed;

pub struct TagDb;

impl TagDb {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(TagDb)
    }

    pub async fn create_tag(
        &self,
        name: &str,
        content: &str,
        guild_id: u64,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let pool = DB_POOL.get().unwrap();
        let conn = pool.get().expect_error("Failed to get database connection");
    
        let table_name = format!("tags_{}", guild_id);

        conn.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {} (
                    name TEXT PRIMARY KEY,
                    content TEXT NOT NULL
                )",
                table_name
            ),
            [],
        )?;
    
        let name = name.to_string();
        let content = content.to_string();
        let table_name_clone = table_name.clone();
    
        task::spawn_blocking(move || {
            let conn = pool.get()?;
            conn.execute(
                &format!(
                    "INSERT INTO {} (name, content) VALUES (?1, ?2)",
                    table_name_clone
                ),
                [&name, &content],
            )?;
            Ok(())
        })
        .await?
    }
    

    pub async fn delete_tag(
        &self,
        name: &str,
        guild_id: u64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let pool = DB_POOL.get().unwrap();

        let table_name = format!("tags_{}", guild_id);
        let name = name.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let modified = conn.execute(&format!("DELETE FROM {} WHERE name = ?1", table_name), [name])?;
            Ok(modified != 0)
        })
        .await?
    }

    pub async fn edit_tag(
        &self,
        name: &str,
        content: &str,
        guild_id: u64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let pool = DB_POOL.get().unwrap();

        let table_name = format!("tags_{}", guild_id);
        let name = name.to_string();
        let content = content.to_string();
        
        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let modified = conn.execute(&format!(
                "UPDATE {} SET content = ?1 WHERE name = ?2", table_name),
                [content, name],
            )?;
            Ok(modified != 0)
        })
        .await?
    }

    pub async fn get_tag(
        &self,
        name: &str,
        guild_id: u64,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let pool = DB_POOL.get().unwrap();

        let table_name = format!("tags_{}", guild_id);
        let name = name.to_string();

        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let mut stmt = conn.prepare(&format!("SELECT content FROM {} WHERE name = ?1", table_name))?;
            let mut rows = stmt.query([name])?;

            if let Some(row) = rows.next()? {
                Ok(Some(row.get(0)?))
            } else {
                Ok(None)
            }
        })
        .await?
    }

    pub async fn get_all_tags(
        &self,
        guild_id: u64,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let pool = DB_POOL.get().unwrap();
        let table_name = format!("tags_{}", guild_id);
    
        task::spawn_blocking(move || {
            let conn = pool.get()?;
            let mut stmt = conn.prepare(&format!("SELECT name FROM {}", table_name))?;
            let rows = stmt.query_map([], |row| row.get(0))?;
    
            let mut tags = Vec::new();
            for tag in rows {
                tags.push(tag?);
            }
    
            Ok(tags)
        })
        .await?
    }    
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum CtxError {
    NotGuild(),
    Discord(String),
}

impl std::error::Error for CtxError {}

impl From<serenity::Error> for CtxError {
    fn from(err: serenity::Error) -> CtxError {
        CtxError::Discord(err.to_string())
    }
}

impl fmt::Display for CtxError {
	fn fmt(
		&self,
		f: &mut fmt::Formatter<'_>,
	) -> fmt::Result {
		match self {
			| CtxError::NotGuild() => write!(f, "Not in Server"),
            | CtxError::Discord(e) => write!(f, "{}", e),
		}
	}
}

pub async fn get_data_and_id(ctx: Context<'_>) -> Result<(&Data, u64), CtxError> {
    let data = ctx.data();

    let id = match ctx.guild_id() {
        Some(id) => id.get(),
        _ => {
            ctx.send(CreateReply::default().embed(create_error_embed("Tags are only avaible in servers"))).await?;
            return Err(CtxError::NotGuild())
        },
    };

    Ok((data, id))
}
