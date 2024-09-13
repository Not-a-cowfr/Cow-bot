from dis import disco

import discord

<<<<<<< Updated upstream
from botSetup import bot
=======
from setup import bot
>>>>>>> Stashed changes

import os
import json

id_file = 'src/data/ticket_ids.json'

class SelectSuggestiontype(discord.ui.View):
    def __init__(self, user: discord.User):
        super().__init__()
        self.user = user

    @discord.ui.select(
        placeholder='Select the type of submission',
        min_values=1,
        max_values=1,
        options=[
            discord.SelectOption(label='Feature', value='feature'),
            discord.SelectOption(label='Improvement', value='improvement'),
            discord.SelectOption(label='Fix', value='fix'),
            discord.SelectOption(label='Idea', value='idea'),
            discord.SelectOption(label='Other', value='other'),
        ]
    )
    async def type_select(self, interaction: discord.Interaction, select: discord.ui.Select):
        submission_type = select.values[0]
        modal = SuggestionModal(submission_type=submission_type, user=interaction.user)
        await interaction.response.send_modal(modal)

class SubmissionReview(discord.ui.View):
    def __init__(self, submitter: discord.User, description: str, submission_type: str):
        super().__init__()
        self.submitter = submitter
        self.description = description
        self.submission_type = submission_type

    @discord.ui.button(label='Accept', style=discord.ButtonStyle.green)
    async def accept_button(self, interaction: discord.Interaction, button: discord.ui.Button):
        for child in self.children:
            child.disabled = True

        ticket_channel_name = f'ticket-suggestion-{self.generate_ticket_id()}'
        guild_id = 1235958749441691699
        category_id = 1282133493765247028

        guild = bot.get_guild(guild_id)
        category = discord.utils.get(guild.categories, id=category_id)

        if not category:
            await interaction.response.send_message('Category not found.', ephemeral=True)
            return

        try:
            channel = await guild.create_text_channel(
                name=ticket_channel_name,
                category=category,
                overwrites={
                    guild.default_role: discord.PermissionOverwrite(read_messages=False),
                    interaction.user: discord.PermissionOverwrite(read_messages=True, send_messages=True),
                }
            )

            await channel.send(
                f'User {self.submitter.mention} submitted a suggestion\n\n'
                f'Type: `{self.submission_type}`\n\n'
                f'Description:\n```{self.description}```'
            )

            await self.submitter.send(
                f'Your suggestion has been accepted. A new ticket has been created: {channel.mention}')

            original_message = interaction.message.content
            await interaction.response.edit_message(
                content=f'{original_message}\n\n**Suggestion accepted by {interaction.user.mention}.**\n'
                        f'**New ticket created:** {channel.mention}',
                view=self
            )
        except Exception as e:
            await interaction.response.send_message(f'Failed to create the ticket. Error: {e}', ephemeral=True)

    @discord.ui.button(label='Reject', style=discord.ButtonStyle.red)
    async def reject_button(self, interaction: discord.Interaction, button: discord.ui.Button):
        for child in self.children:
            child.disabled = True

        modal = RejectSubmissionModal(
            submitter=self.submitter,
            description=self.description,
            original_interaction=interaction
        )
        await interaction.response.send_modal(modal)

        original_message = interaction.message.content
        await interaction.message.edit(
            content=f'{original_message}\n\n**Suggestion rejected by {interaction.user.mention}. \nReason:**'
        )

    def generate_ticket_id(self):
        global id_file
        if os.path.exists(id_file):
            with open(id_file, 'r') as f:
                ticket_ids = json.load(f)
        else:
            ticket_ids = {}

        ticket_ids.setdefault('accepted', 0)
        ticket_ids['accepted'] += 1
        new_ticket_id = ticket_ids['accepted']

        with open(id_file, 'w') as f:
            json.dump(ticket_ids, f)

        return new_ticket_id

class SelectTicketType(discord.ui.View):
    def __init__(self, user: discord.User):
        super().__init__()
        self.user = user

    @discord.ui.select(
        placeholder='Select the type of ticket',
        min_values=1,
        max_values=1,
        options=[
            discord.SelectOption(label='Support', value='support'),
            discord.SelectOption(label='Report', value='report'),
            discord.SelectOption(label='Appeal', value='appeal'),
            discord.SelectOption(label='Suggestion', value='suggestion'),
            discord.SelectOption(label='Other', value= 'other'),
        ]
    )
    async def ticket_type_select(self, interaction: discord.Interaction, select: discord.ui.Select):
        ticket_type = select.values[0]
        ticket_channel_name = f'ticket-{ticket_type}-{self.generate_ticket_id(ticket_type)}'

        guild_id = 1235958749441691699
        category_id = 1282133493765247028

        guild = bot.get_guild(guild_id)
        category = discord.utils.get(guild.categories, id=category_id)

        if not category:
            await interaction.response.send_message('Category not found.', ephemeral=True)
            return

        try:
            channel = await guild.create_text_channel(
                name=ticket_channel_name,
                category=category,
                overwrites={
                    guild.default_role: discord.PermissionOverwrite(read_messages=False),
                    self.user: discord.PermissionOverwrite(read_messages=True, send_messages=True),
                }
            )

            await interaction.response.send_message(f'Ticket created: {channel.mention}', ephemeral=True)

        except Exception as e:
            await interaction.response.send_message(f'Failed to create the ticket channel. Error: {e}', ephemeral=True)

    def generate_ticket_id(self, ticket_type: str):
        global id_file

        if os.path.exists(id_file):
            with open(id_file, 'r') as f:
                ticket_ids = json.load(f)
        else:
            ticket_ids = {}

        if ticket_type not in ticket_ids:
            ticket_ids[ticket_type] = 0

        ticket_ids[ticket_type] += 1
        new_ticket_id = ticket_ids[ticket_type]

        with open(id_file, 'w') as f:
            json.dump(ticket_ids, f)

        return new_ticket_id

class SuggestionModal(discord.ui.Modal):
    def __init__(self, submission_type: str, user: discord.User):
        super().__init__(title=f'Submit a {submission_type.capitalize()} Suggestion')
        self.submission_type = submission_type
        self.user = user
        self.suggestion = discord.ui.TextInput(
            label='Your Suggestion',
            placeholder='Describe your suggestion here...',
            style=discord.TextStyle.long,
            required=True
        )
        self.add_item(self.suggestion)

    async def on_submit(self, interaction: discord.Interaction):
        description = self.suggestion.value

        message = f'User {self.user.mention} submitted a suggestion\n\nType: `{self.submission_type}`:\n\n*{description}*'

        try:
            guild = interaction.guild
            channel_id = 1282171918442565673
            role = discord.utils.get(guild.roles, name='submission ping')

            channel = guild.get_channel(channel_id)
            if not channel:
                await interaction.response.send_message("Submission channel not found.", ephemeral=True)
                return

            review_view = SubmissionReview(
                submitter=self.user,
                description=description,
                submission_type=self.submission_type
            )

            await channel.send(f'<@&{role.id}>\n\n{message}', view=review_view)

            await interaction.response.send_message(
                'Your submission has been sent to the review channel.', ephemeral=True
            )

        except Exception as e:
            await interaction.response.send_message(f'Failed to send the submission. Error: {e}', ephemeral=True)

class RejectSubmissionModal(discord.ui.Modal):
    def __init__(self, submitter: discord.User, description: str, original_interaction: discord.Interaction):
        super().__init__(title='Reject Submission')
        self.submitter = submitter
        self.description = description
        self.original_interaction = original_interaction
        self.reason = discord.ui.TextInput(
            label='Reason for rejection',
            placeholder='Provide the reason here...',
            style=discord.TextStyle.long,
            required=True
        )
        self.add_item(self.reason)

    async def on_submit(self, interaction: discord.Interaction):
        reason = self.reason.value
        try:
            await self.submitter.send(
                f'Your suggestion has been rejected:\n\n*{self.description}*\n\n**Reason:**\n{reason}')

            original_message = self.original_interaction.message.content
            await self.original_interaction.message.edit(
                content=f'{original_message}\n\n**Suggestion rejected by {interaction.user.mention}. Reason:**\n{reason}', view=None
            )

            await interaction.response.send_message('You rejected the suggestion and notified the user.',
                                                          ephemeral=True)
        except Exception as e:
            await interaction.response.send_message(f'Failed to notify the user. Error: {e}', ephemeral=True)