use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use regex::Regex;
use std::time::Instant;

#[derive(Debug)]
struct Politician {
    vorname: String,
    nachname: String,
}

#[derive(Deserialize, Debug)]
struct Article {
    url: String,
    content: String,
    date: String, // Date as a string in ISO 8601 format
}

#[derive(Serialize)]
struct FilteredResult {
    url: String,
    content: String,
    date: String,
    politicians: String, // Store as single-line string
    companies: String,    // Store as single-line string
}

fn load_politicians_from_csv(path: &str) -> Vec<Politician> {
    let file = File::open(path).expect("Unable to open CSV file");
    let reader = BufReader::new(file);
    let mut politicians = Vec::new();

    for line in reader.lines().skip(1) { // Skip header if present
        if let Ok(line) = line {
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() > 2 {
                let vorname = parts[1].trim().to_string();
                let nachname = parts[2].trim().to_string();
                politicians.push(Politician { vorname, nachname });
            }
        }
    }
    politicians
}

fn load_json_file<T: for<'de> serde::Deserialize<'de>>(path: &str) -> T {
    let file = File::open(path).expect("Unable to open file");
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).expect("Unable to parse JSON")
}

fn write_json_file<T: Serialize>(path: &str, data: &T) {
    let serialized = to_string_pretty(data).expect("Unable to serialize data");
    let mut file = File::create(path).expect("Unable to create file");
    file.write_all(serialized.as_bytes()).expect("Unable to write file");
}

fn find_matches(content: &str, patterns: &[Regex]) -> Vec<String> {
    patterns
        .iter()
        .filter_map(|regex| {
            if regex.is_match(content) {
                Some(regex.as_str().to_string())
            } else {
                None
            }
        })
        .collect()
}

fn main() {
    let start_time = Instant::now();

    // Paths to files
    let data_dir = "./../data/";
    let politicians_file = "./../Politician.csv";
    let companies_file = "./../companies.json";

    // Load politicians from CSV
    let politicians = load_politicians_from_csv(politicians_file);
    let companies: Vec<String> = load_json_file(companies_file);

    // Compile patterns for regex matching
    let politician_patterns: Vec<Regex> = politicians
        .iter()
        .map(|p| Regex::new(&regex::escape(&format!("{} {}", p.vorname, p.nachname))).unwrap())
        .collect();

    let company_patterns: Vec<Regex> = companies
        .iter()
        .map(|c| Regex::new(&format!(r"\b{}\b", regex::escape(c))).unwrap())
        .collect();

    // Process each JSON file in the directory
    for entry in fs::read_dir(data_dir).expect("Unable to read directory") {
            let entry = entry.expect("Unable to read entry");
            let path = entry.path();

            if path.extension() == Some("json".as_ref())
                && !path.file_name().unwrap().to_str().unwrap().contains("merkur")
                && !path.file_name().unwrap().to_str().unwrap().contains("rtl")
                && !path.file_name().unwrap().to_str().unwrap().contains("fr")
                && !path.file_name().unwrap().to_str().unwrap().contains("tagesschau")
                && !path.file_name().unwrap().to_str().unwrap().contains("welt")
                && !path.file_name().unwrap().to_str().unwrap().contains("_filtered")
                && !path.file_name().unwrap().to_str().unwrap().contains("bild")
                && !path.file_name().unwrap().to_str().unwrap().contains("faz")
                && !path.file_name().unwrap().to_str().unwrap().contains("focus")
                && !path.file_name().unwrap().to_str().unwrap().contains("spiegel")
                && !path.file_name().unwrap().to_str().unwrap().contains("taz")
            {
                println!("Processing file: {:?}", path);

                // Load articles
                let articles: Vec<Article> = load_json_file(path.to_str().unwrap());

                let mut filtered_results = Vec::new();

                for (count, article) in articles.iter().enumerate() {
                    println!("article: {}", count);
                    
                    let matched_politicians = {
                        let unique_politicians: HashSet<_> = find_matches(&article.content, &politician_patterns).into_iter().collect();
                        unique_politicians.into_iter().collect::<Vec<_>>().join(", ")
                    };
                    
                    let matched_companies: String = {
                        let unique_companies: HashSet<_> = company_patterns
                            .iter()
                            .filter_map(|regex| {
                                if regex.is_match(&article.content) {
                                    Some(regex.as_str().trim_start_matches("\\b").trim_end_matches("\\b").to_string())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        unique_companies.into_iter().collect::<Vec<_>>().join(", ")
                    };
                    
                    if !matched_politicians.is_empty() && !matched_companies.is_empty() {
                        filtered_results.push(FilteredResult {
                            url: article.url.clone(),
                            content: article.content.clone(),
                            date: article.date.clone(),
                            politicians: matched_politicians,
                            companies: matched_companies,
                        });
                    }
                }
                
                if !filtered_results.is_empty() {
                    let output_path = path
                        .with_file_name(format!("{}_filtered.json", path.file_stem().unwrap().to_str().unwrap()))
                        .to_str()
                        .unwrap()
                        .to_string();

                    write_json_file(&output_path, &filtered_results);
                    println!("Filtered results saved to {} with {} matching articles.", output_path, filtered_results.len());
                } else {
                    println!("No matches found in file: {:?}", path);
                }
            }
    }
    let elapsed_time = start_time.elapsed();
    println!("Total execution time: {:.2} seconds", elapsed_time.as_secs_f64());
}
