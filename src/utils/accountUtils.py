import os
import json

# load json file
def loadData(data_file):
    if os.path.exists(data_file):
        with open(data_file, 'r') as file:
            return json.load(file)
    return {}

# save data to JSON file
def saveData(data_file, data):
    with open(data_file, 'w') as file:
        json.dump(data, file, indent=4)
