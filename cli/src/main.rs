use keyword_analyzer_shared::{KeywordAnalyzer, Language, OutputFormat};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (target_path, output_format, language, output_file) = parse_args(&args);

    if !matches!(output_format, OutputFormat::Json) {
        println!("Analyzing files in: {}", target_path);
    }

    match KeywordAnalyzer::analyze_path(target_path, language) {
        Ok(result) => {
            eprintln!("Analysis completed! Found {} files", result.file_count);
            eprintln!("Generating results...\n");

            match KeywordAnalyzer::format_output(&result, output_format, output_file) {
                Ok(message) => {
                    if matches!(output_format, OutputFormat::Plain) {
                        println!("{}", message);
                    } else {
                        println!("{}", message);
                    }
                }
                Err(e) => eprintln!("Error formatting output: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn parse_args(args: &[String]) -> (&str, OutputFormat, Language, Option<String>) {
    let mut target_path = ".";
    let mut output_format = OutputFormat::Plain;
    let mut language = Language::Rust;
    let mut output_file: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--format" | "-f" => {
                if i + 1 < args.len() {
                    output_format = match args[i + 1].as_str() {
                        "json" => OutputFormat::Json,
                        "csv" => OutputFormat::Csv,
                        "html" => OutputFormat::Html,
                        "graph" => OutputFormat::Graph,
                        _ => OutputFormat::Plain,
                    };
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--language" | "-l" => {
                if i + 1 < args.len() {
                    language = match args[i + 1].as_str() {
                        "js" | "ts" | "javascript" | "typescript" => Language::JavaScript,
                        "rust" | "rs" => Language::Rust,
                        "ruby" | "rb" => Language::Ruby,
                        "go" | "golang" => Language::Golang,
                        "python" | "py" => Language::Python,
                        _ => Language::Rust,
                    };
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--output" | "-o" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            arg if !arg.starts_with('-') => {
                target_path = arg;
                i += 1;
            }
            _ => i += 1,
        }
    }

    (target_path, output_format, language, output_file)
}

fn print_help() {
    println!("Multi-Language Keyword Analyzer CLI");
    println!();
    println!("USAGE:");
    println!("    keyword-analyzer [PATH] [OPTIONS]");
    println!();
    println!("ARGS:");
    println!("    <PATH>    Directory, file, or Git URL (GitHub/GitLab) to analyze [default: .]");
    println!();
    println!("OPTIONS:");
    println!("    -l, --language <LANG>    Language to analyze [default: rust] [possible values: rust, rs, js, ts, ruby, rb, go, golang, python, py]");
    println!("    -f, --format <FORMAT>    Output format [default: plain] [possible values: plain, json, csv, html, graph]");
    println!("    -o, --output <FILE>      Output file path (for json, csv, html, graph formats)");
    println!("    -h, --help               Print help information");
    println!();
    println!("EXAMPLES:");
    println!("    keyword-analyzer --language rust");
    println!("    keyword-analyzer --language js src/");
    println!("    keyword-analyzer --language ruby lib/");
    println!("    keyword-analyzer --language go cmd/");
    println!("    keyword-analyzer --language python src/");
    println!("    keyword-analyzer -l ts github.com/microsoft/typescript");
    println!("    keyword-analyzer -l py github.com/psf/requests");
    println!("    keyword-analyzer -l rs gitlab.com/gitlab-org/gitlab");
    println!("    keyword-analyzer -l go https://github.com/golang/go");
    println!(
        "    keyword-analyzer --format json --language python https://github.com/django/django"
    );
    println!("    keyword-analyzer --format json --output results.json --language rust src/");
    println!("    keyword-analyzer --format html --output analysis.html --language js");
    println!("    keyword-analyzer --format graph --output chart.svg --language rust");
    println!("    keyword-analyzer -f csv -o data.csv -l python");
}
