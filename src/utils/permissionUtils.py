import discord

#TODO replace with command to add like "owner" or "sr admin" roles
owner_roles='Not a cow'
#TODO replace with command to add moderator roles
mod_roles='Not a cow', 'Admin', 'Moderator', 'Support Team'

def isMod(user: discord.Member) -> bool:
    global mod_roles
    user_roles = [role.name for role in user.roles]
    return any(role in mod_roles for role in user_roles)

def isCow(user: discord.Member):
    global owner_roles
    user_roles = [role.name for role in user.roles]
    return owner_roles in user_roles