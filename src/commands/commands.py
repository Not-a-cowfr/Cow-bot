import discord
from discord import app_commands
from discord.ext import commands
from botSetup import bot

from commands.submit import TypeSelectionView
from commands.report import ReportReasonModal
from commands.xprates import boosts, crops, FarmingRateOptions
from commands.submit import TicketTypeSelectionView
from utils.playerTracker import player_status, tracked_players, save_tracked_players
from utils.permissionUtils import isMod

import random
import requests

minecraft_username_api = "https://api.mojang.com/users/profiles/minecraft/{}"
mod_roles='Not a cow', 'Admin', 'Moderator', 'Support Team'

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

    @bot.tree.command(name="moo")
    async def moo(interaction: discord.Interaction):
        num = random.randint(1,9999)

        if num == 6969:
            await interaction.response.send_message('meow')
            time.sleep(1.5)
            await interaction.followup.send('...')
            time.sleep(1.5)
            await interaction.followup.send('wait a second cows dont meow')
            await interaction.followup.send(f'<@{cow_id}>')
        else:
            await interaction.response.send_message("moo", ephemeral=True)

    @bot.tree.command(name='track', description='Get notified when a player joins/leaves Hypixel')
    async def track(interaction: discord.Interaction, username: str):
        user_id = str(interaction.user.id)

        response = requests.get(minecraft_username_api.format(username))

        if response.status_code != 200:
            await interaction.response.send_message(f'The username "{username}" is not a valid Minecraft username.',
                                                    ephemeral=True)
            return

        if username in tracked_players:
            if user_id in tracked_players[username]['trackers']:
                await interaction.response.send_message(f'You are already tracking {username}.', ephemeral=True)
                return
            else:
                tracked_players[username]['trackers'].append(user_id)
        else:
            tracked_players[username] = {'trackers': [user_id], 'status': None}

        save_tracked_players()

        await interaction.response.send_message(f'Started tracking {username}.')

    @bot.tree.command(name='untrack', description='Stop tracking a Hypixel player\'s online status')
    async def untrack(interaction: discord.Interaction, username: str):
        user_id = str(interaction.user.id)

        if username in tracked_players and user_id in tracked_players[username]['trackers']:
            tracked_players[username]['trackers'].remove(user_id)

            if not tracked_players[username]['trackers']:
                del tracked_players[username]

            save_tracked_players()
            await interaction.response.send_message(f'Stopped tracking {username}.')
        else:
            await interaction.response.send_message(f'You were not tracking {username}.')


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
        await interaction.response.send_message(embed=embed, view=FarmingView(wisdom, bps))


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
        view = TypeSelectionView(user=interaction.user)
        await interaction.response.send_message(
            'Please select the type of submission from the dropdown menu.',
            ephemeral=True,
            view=view
        )


class Ticket(app_commands.Group):
    allowed_roles = mod_roles

    @app_commands.command(name='create', description='Create a new ticket of a specified type')
    async def create(self, interaction: discord.Interaction):
        view = TicketTypeSelectionView(user=interaction.user)
        await interaction.response.send_message('Please select the type of ticket you want to create:', view=view, ephemeral=True)

    @app_commands.command(name='close', description='Close a ticket with a reason')
    @app_commands.describe(reason='The reason for closing the ticket')
    async def close(self, interaction: discord.Interaction, reason: str):
        if not isMod(interaction.user, self.allowed_roles):
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
        if not isMod(interaction.user, self.allowed_roles):
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
        user = interaction.user
        user_roles = [role.name for role in user.roles]
        if "Not a cow" not in user_roles:
            await interaction.response.send_message('You do not have permission to use this command.', ephemeral=True)
            return

        guild = interaction.guild
        if not guild:
            await interaction.response.send_message('Guild not found.', ephemeral=True)
            return

        closed_ticket_channels = [channel for channel in guild.channels if channel.name.startswith('closed-ticket-')]

        deleted_count = 0
        for channel in closed_ticket_channels:
            try:
                await channel.delete()
                deleted_count += 1
            except Exception as e:
                await interaction.response.send_message(f'Failed to delete channel {channel.name}. Error: {e}',
                                                        ephemeral=True)
                return

        await interaction.response.send_message(f'Deleted {deleted_count} closed ticket(s).')
