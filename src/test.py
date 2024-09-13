import requests


def get_linked_discord(minecraft_username, hypixel_api_key):
    # Get the UUID of the Minecraft username
    mojang_url = f'https://api.mojang.com/users/profiles/minecraft/{minecraft_username}'
    mojang_response = requests.get(mojang_url)

    if mojang_response.status_code != 200:
        return f'Error: Unable to find Minecraft username {minecraft_username}'

    uuid = mojang_response.json().get('id')

    # Get the player data from Hypixel API
    hypixel_url = f'https://api.hypixel.net/player?key={hypixel_api_key}&uuid={uuid}'
    hypixel_response = requests.get(hypixel_url)

    if hypixel_response.status_code != 200:
        return f'Error: Unable to fetch data from Hypixel API for UUID {uuid}'

    player_data = hypixel_response.json().get('player')

    if not player_data:
        return f'Error: No player data found for UUID {uuid}'

    # Extract the linked Discord account
    linked_discord = player_data.get('socialMedia', {}).get('links', {}).get('DISCORD')

    if not linked_discord:
        return f'No linked Discord account found for Minecraft username {minecraft_username}'

    return linked_discord


# Example usage
minecraft_username = 'not_a_cowfr'
hypixel_api_key = '02c9e235-6f51-4ae8-b63f-f9207abe82bc'
linked_discord = get_linked_discord(minecraft_username, hypixel_api_key)
print(f'Linked Discord account for {minecraft_username}: {linked_discord}')