<h1 align="center">
Cow bot
</h1>

Discord bot made for different miscellaneous skyblock utilities

<h2 align="center">
Features
</h2>

- Get a user's mojang info and some quick links - `/get_linked_account`
- Get a user's estimated hypixel uptime - `/uptime`
- Set your own personal custom color for the bot - `/color`
- Feature rich tag system nearly identitical to that of Fire's without any premium subscription

<h2 align="center">
Contribute
</h2>

Check out the [Contributing guide](/CONTRIBUTING.md) for more info on how get things set up

<h2 align="center">
    TODO
</h2>

<h4 align="center">
    Features
</h4>

- [ ] Add stats command to display the amount of tracked players and guilds (tag stats? 👀)
- [ ] Add a command to view the highest uptime of tracked players
- [ ] Use elite api graph endpoint to add collection/skill tracking
    - [ ] Add a command similar to `/uptime` for this
    - [ ] Add a command to view the highest collection/skill gain of tracked players
- [ ] Add total hours and average uptime to `/uptime`
- [ ] Add graph to `/uptime`
- [ ] Make `/link` command to stop using elite api for linked accounts

<h4 align="center">
    Bug fixes
</h4>

- [ ] Fix uptime having duplicated dates

<h4 align="center">
    Improvements
</h4>

- [ ] Optimize uptime command

<h2 align="center">
Credits
</h2>

- **[Serenity](https://github.com/serenity-rs/serenity/)** - The main library used to interface with discord
- **[Poise](https://github.com/serenity-rs/poise)** - The framework that the bot is built on
- **[Hypixel API](https://api.hypixel.net/)** - Used for nearly all player data
- **[Elite API](https://api.elitebot.dev/)** - Used for player farming data
