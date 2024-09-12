import json
import os

import discord
from discord.ext import commands
from botSetup import bot

from commands.commands import standalone_commands, XPRates, Contribute, Ticket
from utils.playerTracker import player_status

bot_token = os.getenv('BOT_TOKEN')

@bot.event
async def on_ready():
    print(f'Bot connected to Discord as `{bot.user}`')

    # add all the command groups
    command_xprates = XPRates(name='xprates', description='Calculate your hourly xp rates for different skills')
    bot.tree.add_command(command_xprates)

    command_contribute = Contribute(name='contribute', description='Details on how to contribute to the bot.')
    bot.tree.add_command(command_contribute)

    command_ticket = Ticket(name='ticket', description='Ticket management commands')
    bot.tree.add_command(command_ticket)

    standalone_commands()
    player_status_tracker = player_status()

    try:
        # sync commands
        synced = await bot.tree.sync()
        print(f'Synced {len(synced)} command(s):')
        for command in synced:
            print(f'  /{command.name}')
    except Exception as e:
        print(f'Error syncing commands: {e}')



bot.run(bot_token)
