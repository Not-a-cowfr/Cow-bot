import requests
import json

def get_linked_discord(minecraft_username, hypixel_api_key):
    mojang_url = f'https://api.mojang.com/users/profiles/minecraft/{minecraft_username}'
    mojang_response = requests.get(mojang_url)

    if mojang_response.status_code != 200:
        return None

    uuid = mojang_response.json().get('id')
    hypixel_url = f'https://api.hypixel.net/player?key={hypixel_api_key}&uuid={uuid}'
    hypixel_response = requests.get(hypixel_url)

    if hypixel_response.status_code != 200:
        return None

    player_data = hypixel_response.json().get('player')
    if not player_data:
        return None

    return player_data.get('socialMedia', {}).get('links', {}).get('DISCORD')