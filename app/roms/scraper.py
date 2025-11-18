# Scrapes entries from Homebrew Hub's (https://hh.gbdev.io/)
# GitHub database and compiles them into a JSON file
# Using GitHub API (reasonably) requires a personal access token 
# without any additional permissions

import requests
from dotenv import dotenv_values
import json

# Load GITHUB_TOKEN from .env
config = dotenv_values(".env");
headers = {
    "Authorization": f"Bearer {config["GITHUB_TOKEN"]}",
}

get_count = 0
def get(url):
    response = requests.get(url, headers=headers)
    response.raise_for_status()
    global get_count
    get_count += 1
    return response.json()

gb_tags = ["gbcompo21", "event:gbcompo23", "gb-showdown-22"]
entry_url_template = "https://api.github.com/repos/gbdev/database/contents/entries/"
entry_tree_url = "https://api.github.com/repos/gbdev/database/git/trees/b1eccaeb0221c97db160354dfe2e8b66ac5d3710"
entries = get(entry_tree_url)["tree"]
roms = {}

for entry in entries:
    entry_files = get(entry_url_template + entry['path'])
    rom_info = None
    
    # Find the game.json file that contains ROM metadata
    for entry_file in entry_files:
        if entry_file['name'] == "game.json":
            rom_info = get(entry_file['download_url'])
        
    if rom_info is None or not "title" in rom_info:
        continue
    # Filter out ROMs that dont contain metadata implying the ROM runs on Game Boy
    if len(set(rom_info.get("tags", [])).intersection(gb_tags)) == 0 and rom_info.get('platform') != "GB":
        continue;

    # Look for default ROM file name in metadata
    rom_file_info = None
    for file in rom_info["files"]:
        # Some ROMs dont specify default parameter if there is only one file
        if len(rom_info["files"]) == 1:
            if file.get("playable"):
                rom_file_info = file
        else:
            if file.get("default") and file.get("playable"):
                rom_file_info = file
    if rom_file_info is None:
        continue;

    # Get info about default ROM file
    rom_file = next(
        (file for file in entry_files if file['name'] == (rom_file_info.get('filename'))),
        None
    )
    if rom_file is None:
        continue
    
    # Save first screenshot URL to use for thumbnail, if it exists
    image_url = ""
    if rom_info["screenshots"] is not None and len(rom_info["screenshots"]) > 0:
        image_url = rom_file["download_url"].rsplit('/', 1)[0] + "/" + rom_info["screenshots"][0];
    
    # Save developers into a list of strings
    # Input field might be string, list of strings or list of objects
    developers = []
    dev_info = rom_info.get("developer", None)
    if type(dev_info) == str:
        developers.append(dev_info)
    elif type(dev_info) == list:
        for dev in dev_info:
            if type(dev) == dict:
                developers.append(dev["name"])
            elif type(dev) == str:
                developers.append(dev)
    game = {
        "developer": ', '.join(developers),
        "typetag": rom_info.get("typetag", ""),
        "download_url": rom_file["download_url"],
        "image_url": image_url,
    }
    roms[rom_info["title"]] = game;
    print(f'Found ROM "{rom_info["title"]}": {game}')

filename = "homebrewhub.json"
# Save list to JSON file
with open(filename, 'w', encoding='utf-8') as f:
    json.dump(roms, f, indent=2, ensure_ascii=False)

print(f"Saved {len(roms)} ROMs to {filename}, used {get_count} API requests (out of 5000 per hour)")
