import discord
import requests
import os
import json
from discord import app_commands
from botSetup import bot

from src.commands.report import ReportReasonModal
from src.commands.submit import SelectTicketType, SelectSuggestiontype
from src.commands.track import tracked_players
from src.commands.xprates import boosts, crops, FarmingRateOptions
from src.commands.link import get_linked_discord
from src.utils.devUtils import wip
from src.utils.jsonDataUtils import loadData, saveData
from src.utils.permissionUtils import isMod, isCow

mod_roles=['Not a cow', 'Admin', 'Moderator', 'Support Team']
data_file = 'src/data/linked_users.json'
tracked_players_file = 'src/data/tracked_players.json'


# singular, ungrouped commands
def standalone_commands():
    report_channel = bot.get_channel(1282333422139740171)
    mod_team_role_id = 1282333563525533697
    @bot.tree.context_menu(name="Report Message")
    async def report_message(interaction: discord.Interaction, message: discord.Message):
        if report_channel:
            report_message = await report_channel.send(
                f"<@&{mod_team_role_id}>\n{interaction.user.mention} reported a message: {message.jump_url}"
            )

            modal = ReportReasonModal(report_message)
            await interaction.response.send_modal(modal)


    @bot.tree.context_menu(name="Report User")
    async def report_user(interaction: discord.Interaction, user: discord.User):
        if report_channel:
            report_message = await report_channel.send(
                f"<@&{mod_team_role_id}>\n{interaction.user.mention} reported {user.mention}."
            )

            modal = ReportReasonModal(report_message)
            await interaction.response.send_modal(modal)


    @bot.tree.context_menu(name='User Info')
    async def user_info(interaction: discord.Interaction, user: discord.Member):
        embed = discord.Embed(title=f'User Info - {user.name}', color=discord.Color.blue())
        embed.add_field(name='ID', value=user.id, inline=True)
        embed.add_field(name='Name', value=user.name, inline=True)
        embed.add_field(name='Discriminator', value=user.discriminator, inline=True)
        embed.add_field(name='Joined At', value=user.joined_at.strftime('%Y-%m-%d %H:%M:%S'), inline=True)
        embed.set_thumbnail(url=user.avatar.url)
        await interaction.response.send_message(embed=embed, ephemeral=True)


    @bot.tree.context_menu(name='Get Linked Account')
    async def get_linked_account(interaction: discord.Interaction, user: discord.Member):
        global data_file
        linked_users = loadData(data_file)
        linked_account = linked_users.get(str(user.id), None)

        if linked_account:
            await interaction.response.send_message(f'{user.name}\'s linked Minecraft account: `{linked_account}`', ephemeral=True)
        else:
            await interaction.response.send_message(f'{user.name} does not have a linked Minecraft account.', ephemeral=True)


    @bot.tree.command(name='track', description='Get notified when a player joins/leaves Hypixel')
    @app_commands.describe(username='Your Minecraft username')
    async def track(interaction: discord.Interaction, username: str):
        user_id = str(interaction.user.id)

        mojang_url = f'https://api.mojang.com/users/profiles/minecraft/{username}'
        mojang_response = requests.get(mojang_url)

        if mojang_response.status_code == 200:
            uuid = mojang_response.json().get('id')
        else:
            await interaction.response.send_message(f'The username "{username}" is not a valid Minecraft username.',
                                                    ephemeral=True)
            return

        global tracked_players_file
        tracked_players = loadData(tracked_players_file)

        if username in tracked_players:
            if user_id in tracked_players[username]['trackers']:
                await interaction.response.send_message(f'You are already tracking {username}.', ephemeral=True)
                return
            else:
                tracked_players[username]['trackers'].append(user_id)
        else:
            tracked_players[username] = {'trackers': [user_id], 'status': None}

        saveData(tracked_players_file, tracked_players)

        await interaction.response.send_message(f'Started tracking `{username}`')


    @bot.tree.command(name='untrack', description='Stop getting notified when a player joins/leaves Hypixel')
    @app_commands.describe(username='Your Minecraft username')
    async def untrack(interaction: discord.Interaction, username: str):
        user_id = str(interaction.user.id)

        global tracked_players_file
        tracked_players = loadData(tracked_players_file)

        if username not in tracked_players or user_id not in tracked_players[username]['trackers']:
            await interaction.response.send_message(f'You are not tracking {username}.', ephemeral=True)
            return

        tracked_players[username]['trackers'].remove(user_id)

        if not tracked_players[username]['trackers']:
            del tracked_players[username]

        saveData(tracked_players_file, tracked_players)

        await interaction.response.send_message(f'Stopped tracking `{username}`')

    @bot.tree.command(name='link', description='Link your Discord ID with your Minecraft username')
    @app_commands.describe(username='Your Minecraft username')
    async def link(interaction: discord.Interaction, username: str):
        user_id = str(interaction.user.id)
        data_file = 'src/data/linked_users.json'
        linked_users = loadData(data_file)

        if user_id in linked_users:
            await interaction.response.send_message(
                'You have already linked your Discord ID with a Minecraft username.', ephemeral=True)
            return

        hypixel_api_key = os.getenv('API_KEY')
        linked_discord = get_linked_discord(username, hypixel_api_key)
        if linked_discord is None:
            await interaction.response.send_message(
                'No linked Discord account found for the provided Minecraft username.', ephemeral=True)
            return

        if linked_discord != interaction.user.name:
            await interaction.response.send_message('You do not have access to link this Minecraft username.',
                                                    ephemeral=True)
            return

        linked_users[user_id] = username
        saveData(data_file, linked_users)

        await interaction.response.send_message(
            f'Your Discord ID has been linked with the Minecraft username: `{username}`', ephemeral=True)


    @bot.tree.command(name='unlink', description='Unlink your Discord ID from your Minecraft username')
    async def unlink(interaction: discord.Interaction):
        user_id = str(interaction.user.id)
        global data_file
        linked_users = loadData(data_file)

        if user_id in linked_users:
            del linked_users[user_id]
            saveData(data_file, linked_users)
            await interaction.response.send_message('Your Discord ID has been unlinked from your Minecraft username.',
                                                    ephemeral=True)
        else:
            await interaction.response.send_message('You do not have a linked Minecraft username.', ephemeral=True)


    @bot.tree.command(name='ping', description='Check the bot\'s latency')
    async def ping(interaction: discord.Interaction):
        latency = round(bot.latency * 1000)
        await interaction.response.send_message(f'Pong! Latency is {latency}ms', ephemeral=True)



# command groups
class XPRates(app_commands.Group):
    @app_commands.command(name='farming', description='Calculate your estimated hourly XP for Farming')
    async def farming(self, interaction: discord.Interaction, wisdom: float, bps: float):
        wisdom_multiplier = 1 + wisdom / 100
        boost_multiplier = boosts['No Event Boost']
        derpy_multiplier = 1.0
        total_multiplier = wisdom_multiplier * boost_multiplier * derpy_multiplier
        embed = discord.Embed(
            title='Hourly Farming XP Rates',
            description=(
                f'**Total XP Multiplier:** `{round(total_multiplier, 2)}x`\n\n'
                f'**Breakdown:**\n'
                f'- **Wisdom ({wisdom}%):** `{round(wisdom_multiplier, 2)}x`\n'
                f'- **Boost:** `{boost_multiplier}x`\n'
                f'- **Derpy:** `{derpy_multiplier}x`'
            ),
            colour=0xbabd00
        )
        embed.set_footer(
            text='Calculator by not_a_cow',
            icon_url='https://cdn.discordapp.com/avatars/771778437425397820/da545aab4b93fadbba8e58662fbb5b98.webp?'
        )
        crops_list = '\n'.join([f'**{crop}**' for crop in crops.keys()])
        xp_rates_list = '\n'.join([
            f'`{round(bps * 3600 * wisdom_multiplier * boost_multiplier * derpy_multiplier * value):,}`'
            for value in crops.values()
        ])
        embed.add_field(name='Crops', value=crops_list, inline=True)
        embed.add_field(name='XP per hour', value=xp_rates_list, inline=True)
        await interaction.response.send_message(embed=embed, view=FarmingRateOptions(wisdom, bps))


class Contribute(app_commands.Group):
    @app_commands.command(name='info', description='Details on how to contribute to the bot.')
    async def info(self, interaction: discord.Interaction):
        message = (
            '## Want to contribute? Submit your idea with `/contribute submit` and wait for a response\n'
            'The requirement for contributor role is to make a minimum of one good feature or multiple small improvements.\n'
            '-# Note that the discord bot is made with discord.py, not JavaScript or any other language.'
        )
        await interaction.response.send_message(message, ephemeral=True)

    @app_commands.command(name='submit', description='Submit a feature, improvement, fix, or other for the bot.')
    async def submit(self, interaction: discord.Interaction):
        view = SelectSuggestiontype(user=interaction.user)
        await interaction.response.send_message(
            'Please select the type of submission from the dropdown menu.',
            ephemeral=True,
            view=view
        )


class Ticket(app_commands.Group):
    allowed_roles = mod_roles

    @app_commands.command(name='create', description='Create a new ticket of a specified type')
    async def create(self, interaction: discord.Interaction):
        view = SelectTicketType(user=interaction.user)
        await interaction.response.send_message('Please select the type of ticket you want to create:', view=view, ephemeral=True)

    @app_commands.command(name='close', description='Close a ticket with a reason')
    @app_commands.describe(reason='The reason for closing the ticket')
    async def close(self, interaction: discord.Interaction, reason: str):
        if not isMod(interaction.user):
            await interaction.response.send_message('You do not have permission to use this command.', ephemeral=True)
            return

        if not interaction.channel.name.startswith('ticket'):
            await interaction.response.send_message('This command can only be used in a ticket channel.',
                                                    ephemeral=True)
            return

        if interaction.channel.name.startswith('closed-'):
            await interaction.response.send_message('This ticket is already closed.', ephemeral=True)
            return

        try:
            new_channel_name = f'closed-{interaction.channel.name}'
            await interaction.channel.edit(name=new_channel_name)

            overwrites = {
                interaction.guild.default_role: discord.PermissionOverwrite(send_messages=False),
            }
            for role_name in self.allowed_roles:
                role = discord.utils.get(interaction.guild.roles, name=role_name)
                if role:
                    overwrites[role] = discord.PermissionOverwrite(send_messages=True)

            for member in interaction.channel.members:
                overwrites[member] = discord.PermissionOverwrite(read_messages=True, send_messages=False)

            category_id = 1282150713065078854
            category = discord.utils.get(interaction.guild.categories, id=category_id)
            if not category:
                await interaction.response.send_message('Category not found.', ephemeral=True)
                return

            await interaction.channel.edit(category=category, overwrites=overwrites)

            await interaction.channel.send(
                f'Ticket has been closed by {interaction.user.mention} for the following reason: `{reason}`')

            await interaction.response.send_message(
                f'Ticket has been closed and moved to the closed category as {new_channel_name}.', ephemeral=True)
        except Exception as e:
            await interaction.response.send_message(f'Failed to close the ticket. Error: {e}', ephemeral=True)

    @app_commands.command(name='add', description='Add a user to the ticket')
    @app_commands.describe(user='The user to add to a ticket')
    async def add(self, interaction: discord.Interaction, user: discord.Member):
        if not isMod(interaction.user):
            await interaction.response.send_message('You do not have permission to use this command.', ephemeral=True)
            return

        if not interaction.channel.name.startswith('ticket'):
            await interaction.response.send_message('This command can only be used in a ticket channel.', ephemeral=True)
            return

        try:
            await interaction.channel.set_permissions(user, read_messages=True, send_messages=True)
            await interaction.response.send_message(f'{user.mention} has been added to the ticket.', ephemeral=True)

            await interaction.channel.send(f'{user.mention} has been added to the ticket by {interaction.user.mention}.')
        except Exception as e:
            await interaction.response.send_message(f'Failed to add {user.mention} to the ticket. Error: {e}', ephemeral=True)

    @app_commands.command(name='purge', description='Purge all closed tickets.')
    async def purge(self, interaction: discord.Interaction):
        if not isCow(interaction.user):
            await interaction.response.send_message('You do not have permission to use this command.', ephemeral=True)
            return

        guild = interaction.guild
        closed_ticket_channels = [channel for channel in guild.channels if channel.name.startswith('closed-ticket-')]

        deleted_count = 0
        for channel in closed_ticket_channels:
            try:
                await channel.delete()
                deleted_count += 1
            except Exception as e:
                await interaction.response.send_message(f'Failed to delete channel {channel.name}. Error: {e}', ephemeral=True)
                return

        if interaction.channel:
            await interaction.response.send_message(f'Deleted {deleted_count} closed ticket(s).')
