import discord
from dotenv import load_dotenv
from discord.ext import tasks
from discord.ui import Button, View

from botSetup import bot
from src.utils.jsonDataUtils import loadData, saveData

import os
import time
import aiohttp
import json

load_dotenv()
hypixel_api_key = os.getenv('API_KEY')

tracked_players_file = 'src/data/tracked_players.json'
tracked_players = loadData(tracked_players_file)

@tasks.loop(minutes=1)
async def check_player_status():
    channel_id = 1282579021988892733
    channel = bot.get_channel(channel_id)

    if channel:
        await channel.send(f"Checking tracked players status <t:{int(time.time())}>")
    else:
        print(f"Channel with ID {channel_id} not found.")

    async with aiohttp.ClientSession() as session:
        for username, user_data in tracked_players.items():
            prev_status = user_data.get('status', None)

            try:
                uuid = await get_uuid(session, username)
                if uuid is None:
                    continue

                url = f'https://api.hypixel.net/status?key={hypixel_api_key}&uuid={uuid}'
                async with session.get(url) as response:
                    if response.status == 200:
                        result = await response.json()
                        online_status = result['session']['online']

                        if prev_status is not None and online_status != prev_status:
                            status_text = "online" if online_status else "offline"
                            for user_id in user_data['trackers']:
                                user = await bot.fetch_user(int(user_id))

                                untrack_button = Button(label=f"Stop Tracking {username}", style=discord.ButtonStyle.danger)

                                async def untrack(interaction: discord.Interaction):
                                    await untrack_player(interaction, username, user_id)
                                    await interaction.response.send_message(f"You are no longer tracking {username}.", ephemeral=True)

                                untrack_button.callback = untrack

                                view = View()
                                view.add_item(untrack_button)
                                await user.send(f'`{username}` is now {status_text}.', view=view)

                        tracked_players[username]['status'] = online_status
                        saveData(tracked_players_file, tracked_players)

            except Exception as e:
                print(f"Error checking {username}: {e}")
                pass

async def untrack_player(interaction, username, user_id):
    if username in tracked_players and user_id in tracked_players[username]['trackers']:
        tracked_players[username]['trackers'].remove(user_id)
        if not tracked_players[username]['trackers']:
            del tracked_players[username]
        saveData(tracked_players_file, tracked_players)

async def get_uuid(session, username):
    url = f'https://api.mojang.com/users/profiles/minecraft/{username}'
    async with session.get(url) as response:
        if response.status == 200:
            data = await response.json()
            return data['id']
        return None