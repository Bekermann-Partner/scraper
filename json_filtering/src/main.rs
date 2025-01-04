use serde::{Deserialize, Serialize};
use serde_json::{to_string_pretty};
use std::fs::{self, File};
use std::io::{Write};
use regex::Regex;
use std::time::Instant;
use std::collections::HashSet;

#[derive(Deserialize, Debug)]
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

fn load_json_file<T: for<'de> serde::Deserialize<'de>>(path: &str) -> T {
    let file = std::fs::File::open(path).expect("Unable to open file");
    let reader = std::io::BufReader::new(file);
    let data: T = serde_json::from_reader(reader).expect("Unable to parse JSON");
    data
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

    // Paths to JSON files
    let data_dir = "./../data";
    let politicians_file = "./../politicians.json";
    let companies_file = "./../companies.json";

    // Load politicians and companies
    let politicians: Vec<Politician> = load_json_file(politicians_file);
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
            && !path.file_name().unwrap().to_str().unwrap().contains("filtered")
            && !path.file_name().unwrap().to_str().unwrap().contains("spiegel")
            && !path.file_name().unwrap().to_str().unwrap().contains("ndr")
            && !path.file_name().unwrap().to_str().unwrap().contains("faz")
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
                    // Find matches and store unique company names
                    let unique_companies: HashSet<_> = company_patterns
                        .iter()
                        .filter_map(|regex| {
                            if regex.is_match(&article.content) {
                                // Remove regex boundaries for output
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
                    .with_file_name(format!(
                        "{}_filtered.json",
                        path.file_stem().unwrap().to_str().unwrap()
                    ))
                    .to_str()
                    .unwrap()
                    .to_string();

                write_json_file(&output_path, &filtered_results);
                println!(
                    "Filtered results saved to {} with {} matching articles.",
                    output_path,
                    filtered_results.len()
                );
            } else {
                println!("No matches found in file: {:?}", path);
            }
        }
    }
    let elapsed_time = start_time.elapsed();
    println!(
        "Total execution time: {:.2} seconds",
        elapsed_time.as_secs_f64()
    );
}
