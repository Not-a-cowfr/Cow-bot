

#TODO update so that wip() itself sends the message
#like this or smth:
"""
async def wip(type, interaction)
    await interaction.response.send_message(f'{type} under heavy devlopment :construction_site::man_construction_worker:')
"""
# and then it should work like this:
"""
async def modRoles(interaction: discord.Interaction):
    wip('command', interaction)
"""
def wip(type):
    return(f'{type} under heavy devlopment :construction_site::man_construction_worker:')