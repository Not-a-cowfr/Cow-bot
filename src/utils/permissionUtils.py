import discord

cow_id = 771778437425397820
mod_roles='Not a cow', 'Admin', 'Moderator', 'Support Team'

def isMod(user: discord.Member, mod_roles: set) -> bool:
    user_roles = [role.name for role in user.roles]
    return any(role in mod_roles for role in user_roles)

def isCow(user: discord.Member, cow_id: set):
    user_roles = [role.name for role in user.roles]
    return (cow_id in user_roles)