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
- Feature rich tag system nearly identitical to that of Fire's

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
- [ ] Use elite api grapoh endpoint to add collection/skill tracking
    - [ ] Add a command similar to `/uptime` for this
    - [ ] Add a command to view the highest collection/skill gain of tracked players
- [ ] Add total hours and average uptime to `/uptime`

<h4 align="center">
    Bug fixes
</h4>

- [ ] Fix uptime having duplicated dates

<h4 align="center">
    Improvements
</h4>

- [ ] Stagger uptime updater to avoid api key limits

<h2 align="center">
Credits
</h2>

- **[Serenity](https://github.com/serenity-rs/serenity/)** - The main library used to interface with discord
- **[Poise](https://github.com/serenity-rs/poise)** - The framework that the bot is built on
- **[Hypixel API](https://api.hypixel.net/)** - Used for nearly all player data
- **[Elite API](https://api.elitebot.dev/)** - Used for player farming data
