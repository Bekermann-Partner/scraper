# License: Add the appropriate license here (e.g., Apache 2.0, BSD, LGPL, GPL).

"""Analyze JSON files in a directory for occurrences of politicians and companies.

This module reads JSON files in a specified directory, identifies if the
content contains mentions of both a politician and a company, and then
saves filtered results into new JSON files. Each result is saved in a
separate file named after the original file with a `_reduced` suffix.

Typical usage example:

    analyze_json_files(data_directory, politicians_json, companies_json)

Attributes:
    data_directory (str): Path to the directory containing input JSON files.
    politicians_json (str): Path to the JSON file with politician names.
    companies_json (str): Path to the JSON file with company names.
"""

import os
import json
from typing import List
import re

def load_json(file_path: str):
    """Loads a JSON file and returns its content.

    Args:
        file_path (str): Path to the JSON file.

    Returns:
        dict or list: Parsed JSON content.
    """
    with open(file_path, 'r', encoding='utf-8') as file:
        return json.load(file)

def write_json(file_path: str, data):
    """Writes data to a JSON file, overwriting its contents.

    Args:
        file_path (str): Path to the JSON file.
        data: Data to write.
    """
    with open(file_path, 'w', encoding='utf-8') as file:
        json.dump(data, file, ensure_ascii=False, indent=4)


def find_matches(content: str, keywords: List[str]) -> List[str]:
    """Finds all keywords that appear as whole words in the content.

    Args:
        content (str): The text content to search.
        keywords (List[str]): List of keywords to search for.

    Returns:
        List[str]: A list of keywords found in the content.
    """
    return [
        keyword for keyword in keywords
        if re.search(rf'\b{re.escape(keyword)}\b', content, re.IGNORECASE)
    ]

def analyze_json_files(data_dir: str, politicians_file: str, companies_file: str):
    """Analyzes JSON files in the specified directory and saves filtered data.

    Each JSON file is processed independently. If both a politician and a company
    name are found in the content of a JSON file, the relevant data is saved
    to a new JSON file named `<original_filename>_filtered`.

    Args:
        data_dir (str): Directory containing the JSON files to analyze.
        politicians_file (str): Path to the JSON file containing politicians.
        companies_file (str): Path to the JSON file containing companies.
    """
    # Load the list of politicians and companies
    politicians_data = load_json(politicians_file)
    politicians = [f"{p['vorname']} {p['nachname']}" for p in politicians_data]
    companies = load_json(companies_file)

    # Iterate through all JSON files in the data directory
    for file_name in os.listdir(data_dir):
        if file_name.endswith('.json') and 'filtered' not in file_name:
            file_path = os.path.join(data_dir, file_name)
            output_file_path = os.path.join(data_dir, f"{file_name.replace('.json', '')}_filtered.json")

            # Initialize a list to hold the filtered results
            filtered_results = []

            try:
                # Load JSON content
                file_data = load_json(file_path)

                for i, entry in enumerate(file_data):
                    print(f"artikel: {i}")
                    url = entry['url']
                    content = entry['content']

                    # Find matches for politicians and companies
                    matched_politicians = find_matches(content, politicians)
                    matched_companies = find_matches(content, companies)

                    # If both politicians and companies are found, add to results
                    if matched_politicians and matched_companies:
                        filtered_results.append({
                            "url": url,
                            "content": content,
                            "politicians": ", ".join(matched_politicians),
                            "companies": ", ".join(matched_companies),
                        })

                # Write all filtered results to the output file at once
                if filtered_results:
                    write_json(output_file_path, filtered_results)

            except Exception as e:
                print(f"Error processing file {file_name}: {e}")

if __name__ == "__main__":
    # Define paths
    data_directory = "./data"  # Directory containing the input JSON files
    politicians_json = "politicians.json"  # Path to the JSON file with politician names
    companies_json = "companies.json"  # Path to the JSON file with company names

    # Run the analysis
    analyze_json_files(data_directory, politicians_json, companies_json)
