### Setting Up
- Fill out required environment variables
    - Create a copy of [.env.example](.env.example) and rename it to `.env`
    - Get your hypixel api key from the [developer dashboard](https://developer.hypixel.net/)
    - Create a [discord bot](https://discord.com/developers/docs/resources/application) and copy its private token

### How to add a command
- Add a file ending with `_command` in `src/commands/`
- Create a function in that file with the sasme name as the file, excluding the `_command`
- Make sure your command include poise macro to define what kind of command it is, and takes a context param
```rust
use crate::{Context, Error};

#[poise::command(slash_command)]
pub async fn color(
	ctx: Context<'_>,
) -> Result<(), Error> {
    Ok(())
}
```
This will now be automatically generated as a command upon running