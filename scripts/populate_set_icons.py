#!/usr/bin/env python3
import os
import time
import requests

# URL to get the list of sets from Scryfall
SETS_URL = 'https://api.scryfall.com/sets'
# Define the output directory relative to the script's location
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
OUTPUT_DIR = os.path.join(SCRIPT_DIR, '..', 'assets', 'set_icons')
# Scryfall's API limit: no more than 5 requests per second (i.e. wait at least 0.2 seconds between requests)
REQUEST_DELAY = 0.2

def download_set_icons():
    # Ensure the output directory exists (it will create all intermediate directories as needed)
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    print("Fetching list of Magic: The Gathering sets from Scryfall...")
    response = requests.get(SETS_URL)
    if response.status_code != 200:
        print(f"Error fetching sets: HTTP {response.status_code}")
        return

    data = response.json()
    sets = data.get('data', [])
    print(f"Found {len(sets)} sets.")

    # Loop through each set in the response data
    for set_obj in sets:
        set_name = set_obj.get('name')
        set_code = set_obj.get('code')
        icon_url = set_obj.get('icon_svg_uri')

        if not icon_url:
            print(f"Set '{set_name}' ({set_code}) does not have an icon URL, skipping.")
            continue

        # Create a filename for the icon using the set code (SVG file)
        filename = f"{set_code}.svg"
        filepath = os.path.join(OUTPUT_DIR, filename)

        # Skip downloading if the file already exists
        if os.path.exists(filepath):
            print(f"File '{filename}' already exists. Skipping download for set '{set_name}'.")
            continue

        print(f"Downloading icon for set '{set_name}' from {icon_url}...")
        icon_response = requests.get(icon_url)
        if icon_response.status_code == 200:
            with open(filepath, 'wb') as f:
                f.write(icon_response.content)
            print(f"Saved '{filename}' to '{OUTPUT_DIR}'.")
        else:
            print(f"Error downloading icon for '{set_name}': HTTP {icon_response.status_code}")

        # Respect the 5 requests per second limit
        time.sleep(REQUEST_DELAY)

if __name__ == "__main__":
    download_set_icons()
