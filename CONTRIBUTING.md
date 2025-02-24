### Prerequisites
- Git
- Rust
- [msvc build tools](https://visualstudio.microsoft.com/downloads/?q=build+tools)
    - You can also use this [gist](https://gist.github.com/mmozeiko/7f3162ec2988e81e56d5c4e22cde9977) if you just need
      the build tools for rust
- sqlite3
- MongoDB server
- A Discord bot
- Hypixel api key

### Setting Up
- Clone the repository `git clone https://github.com/Not-a-cowfr/Cow-bot.git`
- Fill out required environment variables
    - Create a copy of [.env.example](.env.example) and rename it to `.env`
    - Get your hypixel api key from the [developer dashboard](https://developer.hypixel.net/)
    - Create a [discord bot](https://discord.com/developers/applications) and copy its private token
    - Setup a MongoDb server however you like and copy the url
- Run `cargo run --profile dev` or use [Code Runner](https://marketplace.visualstudio.com/items?itemName=formulahendry.code-runner) VSCode extension and click run

### How to add a command
- Add a file ending with `_command` in `src/commands/`
- Create a function in that file with the sasme name as the file, excluding the `_command`
- Make sure your command include poise macro to define what kind of command it is, and takes a context param
```rust
use crate::{Context, Error};

/// slash command descriptions are made like this with 3 /
#[poise::command(slash_command)]
pub async fn color(
	ctx: Context<'_>,
) -> Result<(), Error> {
    // do whatever you want, I recommend checking out the poise and serenity docs
    Ok(())
}
```
This will now be automatically generated as a command upon running thanks to [build.rs](build.rs)