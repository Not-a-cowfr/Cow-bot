import discord

boosts = {
    'No Event Boost': 1.0,
    '5% Event Boost': 1.05,
    '10% Event Boost': 1.10,
    '15% Event Boost': 1.15,
    '20% Event Boost': 1.20,
    '2x Multiplier': 2.0
}

crops = {
    'Cactus': 4, 'Carrot': 4, 'Cocoa Beans': 4, 'Melon': 4, 'Mushroom': 6,
    'Nether Wart': 4, 'Potato': 4, 'Pumpkin': 4.5, 'Sugar Cane': 4, 'Wheat': 4
}

class FarmingRateOptions(discord.ui.View):
    def __init__(self, wisdom: float, bps: float):
        super().__init__()
        self.wisdom = wisdom
        self.bps = bps
        self.derpy_active = False
        self.boost = boosts['No Event Boost']

    @discord.ui.button(label='Derpy', style=discord.ButtonStyle.red)
    async def derpy_button(self, interaction: discord.Interaction, button: discord.ui.Button):
        self.derpy_active = not self.derpy_active
        button.style = discord.ButtonStyle.green if self.derpy_active else discord.ButtonStyle.red
        await self.update_embed(interaction)

    @discord.ui.select(
        placeholder='Choose an Event Boost',
        min_values=1,
        max_values=1,
        options=[
            discord.SelectOption(label='No Event Boost', value='No Event Boost'),
            discord.SelectOption(label='5% Event Boost', value='5% Event Boost'),
            discord.SelectOption(label='10% Event Boost', value='10% Event Boost'),
            discord.SelectOption(label='15% Event Boost', value='15% Event Boost'),
            discord.SelectOption(label='20% Event Boost', value='20% Event Boost'),
            discord.SelectOption(label='2x Multiplier', value='2x Multiplier'),
        ]
    )
    async def boost_select(self, interaction: discord.Interaction, select: discord.ui.Select):
        selected_boost_label = select.values[0]
        self.boost = boosts[selected_boost_label]
        await self.update_embed(interaction)

    async def update_embed(self, interaction: discord.Interaction):
        wisdom_multiplier = 1 + self.wisdom / 100
        boost_multiplier = self.boost
        derpy_multiplier = 1.5 if self.derpy_active else 1.0
        total_multiplier = wisdom_multiplier * boost_multiplier * derpy_multiplier
        updated_xp_rates_list = '\n'.join([
            f'`{round(self.bps * 3600 * wisdom_multiplier * boost_multiplier * derpy_multiplier * value):,}`'
            for value in crops.values()
        ])
        embed = discord.Embed(
            title='Hourly Farming XP Rates',
            description=(
                f'**Total XP Multiplier:** `{round(total_multiplier, 2)}x`\n'
                f'**Breakdown:**\n'
                f'- **Wisdom ({self.wisdom}%):** `{round(wisdom_multiplier, 2)}x`\n'
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
        embed.add_field(name='Crops', value=crops_list, inline=True)
        embed.add_field(name='XP per hour', value=updated_xp_rates_list, inline=True)
        await interaction.response.edit_message(embed=embed, view=self)
