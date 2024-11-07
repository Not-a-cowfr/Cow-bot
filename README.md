<h1 align="center">
Cow bot
</h1>

Discord bot made for different miscellaneous skyblock utilities

<h2 align="center">
Features
</h2>

- Get a user's mojang info - `/get_linked_account`
- Get a user's estimated hypixel uptime - `/uptime`
- Set your custom color for the bot - `/color`

<h2 align="center">
Set up
</h2>

### Prerequisites
- Git
- Rust
- [msvc build tools](https://visualstudio.microsoft.com/downloads/?q=build+tools)
  - You can also use this [gist](https://gist.github.com/mmozeiko/7f3162ec2988e81e56d5c4e22cde9977) if you just need the build tools for rust
- A Discord bot
- [Hypixel api key](https://developer.hypixel.net/)

### Steps
- Clone the repository
- Add required environment variables
  - `DISCORD_TOKEN` - Your discord bot's token
  - `API_KEY` - Your hypixel api key
- Run `cargo run` in the root directory

<h2 align="center">
Credits
</h2>

- **[Serenity](https://github.com/serenity-rs/serenity/)** - The main library used to interface with discord
- **[Poise](https://github.com/serenity-rs/poise)** - The framework that the bot is built on
- **[Hypixel API](https://api.hypixel.net/)** - Used for nearly all player data
- **[Elite API](https://api.elitebot.dev/)** - Used for player farming data
