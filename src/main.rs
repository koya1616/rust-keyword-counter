use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

const RUST_KEYWORDS: &[&str] = &[
    // Primitive types
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "str", // Control flow
    "if", "else", "match", "break", "continue", "loop", "while", "for",
    // Type definition
    "struct", "enum", "trait", "type", // Function-related
    "fn", "return", "move", // Visibility/Mutability
    "pub", "mut", "const", "static", // Module/Scope
    "mod", "use", "crate", "extern", "super", "self", "Self", // Concurrency
    "async", "await", // Other keywords
    "as", "in", "let", "ref", "where", "unsafe", "true", "false",
    // Reserved keywords
    "abstract", "become", "box", "do", "final", "macro", "override", "priv", "try", "typeof",
    "unsized", "virtual", "yield",
];

fn main() {
    let args: Vec<String> = env::args().collect();
    let (target_path, output_format) = parse_args(&args);

    let actual_path = if is_github_url(&target_path) {
        match clone_github_repo(&target_path) {
            Ok(temp_path) => temp_path,
            Err(e) => {
                eprintln!("Error cloning repository: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        target_path.to_string()
    };

    if !matches!(output_format, OutputFormat::Json) {
        println!("Analyzing Rust files in: {}", actual_path);
    }

    let mut total_counts: HashMap<String, usize> = HashMap::new();
    let mut file_count = 0;

    match analyze_directory(&actual_path, &mut total_counts, &mut file_count) {
        Ok(_) => {
            eprintln!("Analysis completed! Found {} files", file_count);
            eprintln!("Generating results...\n");
            print_results(&total_counts, file_count, output_format);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    // Clean up temporary directory if it was a GitHub URL
    if is_github_url(&target_path) {
        eprintln!("Cleaning up temporary directory...");
        let _ = fs::remove_dir_all(&actual_path);
    }
}

#[derive(Clone, Copy)]
enum OutputFormat {
    Plain,
    Json,
    Csv,
}

fn parse_args(args: &[String]) -> (&str, OutputFormat) {
    let mut target_path = ".";
    let mut output_format = OutputFormat::Plain;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--format" | "-f" => {
                if i + 1 < args.len() {
                    output_format = match args[i + 1].as_str() {
                        "json" => OutputFormat::Json,
                        "csv" => OutputFormat::Csv,
                        _ => OutputFormat::Plain,
                    };
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

    (target_path, output_format)
}

fn print_help() {
    println!("Rust Keyword Analyzer");
    println!();
    println!("USAGE:");
    println!("    app [PATH] [OPTIONS]");
    println!();
    println!("ARGS:");
    println!("    <PATH>    Directory, file, or GitHub URL to analyze [default: .]");
    println!();
    println!("OPTIONS:");
    println!("    -f, --format <FORMAT>    Output format [default: plain] [possible values: plain, json, csv]");
    println!("    -h, --help               Print help information");
    println!();
    println!("EXAMPLES:");
    println!("    app");
    println!("    app src/");
    println!("    app https://github.com/rust-lang/rust");
    println!("    app --format json https://github.com/tokio-rs/tokio");
}

fn is_github_url(input: &str) -> bool {
    input.starts_with("https://github.com/") || input.starts_with("http://github.com/")
}

fn clone_github_repo(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let temp_dir = format!("/tmp/rust_analyzer_{}", std::process::id());

    eprintln!("Cloning repository: {}", url);
    eprintln!("Target directory: {}", temp_dir);

    let output = Command::new("git")
        .args(&["clone", "--depth", "1", url, &temp_dir])
        .output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to clone repository: {}", error_msg).into());
    }

    eprintln!("Repository cloned successfully");

    Ok(temp_dir)
}

fn analyze_directory(
    path: &str,
    total_counts: &mut HashMap<String, usize>,
    file_count: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(path);

    if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
        eprintln!("Analyzing file: {}", path.display());
        analyze_file(path, total_counts)?;
        *file_count += 1;
        eprintln!("Files processed: {}", file_count);
    } else if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() && !should_skip_dir(&entry_path) {
                eprintln!("Entering directory: {}", entry_path.display());
                analyze_directory(entry_path.to_str().unwrap(), total_counts, file_count)?;
            } else if entry_path.extension().map_or(false, |ext| ext == "rs") {
                eprintln!("Analyzing file: {}", entry_path.display());
                analyze_file(&entry_path, total_counts)?;
                *file_count += 1;
                eprintln!("Files processed: {}", file_count);
            }
        }
    }

    Ok(())
}

fn should_skip_dir(path: &Path) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        matches!(name, "target" | ".git" | "node_modules")
    } else {
        false
    }
}

fn analyze_file(
    path: &Path,
    total_counts: &mut HashMap<String, usize>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let file_counts = count_keywords(&content);

    for (keyword, count) in file_counts {
        *total_counts.entry(keyword).or_insert(0) += count;
    }

    Ok(())
}

fn count_keywords(content: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    // Simple tokenization - split on non-alphanumeric characters
    for word in content.split(|c: char| !c.is_alphanumeric() && c != '_') {
        if RUST_KEYWORDS.contains(&word) {
            *counts.entry(word.to_string()).or_insert(0) += 1;
        }
    }

    counts
}

fn print_results(counts: &HashMap<String, usize>, file_count: usize, format: OutputFormat) {
    let mut sorted_counts: Vec<_> = counts.iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(a.1));

    match format {
        OutputFormat::Plain => print_plain(&sorted_counts, file_count),
        OutputFormat::Json => print_json(&sorted_counts, file_count),
        OutputFormat::Csv => print_csv(&sorted_counts, file_count),
    }
}

fn print_plain(sorted_counts: &[(&String, &usize)], file_count: usize) {
    println!("\n=== Rust Keyword Analysis Results ===");
    println!("Files analyzed: {}", file_count);
    println!(
        "Total keywords found: {}\n",
        sorted_counts
            .iter()
            .map(|(_, count)| **count)
            .sum::<usize>()
    );

    for (keyword, count) in sorted_counts {
        if **count > 0 {
            println!("{:12} : {}", keyword, count);
        }
    }
}

fn print_json(sorted_counts: &[(&String, &usize)], file_count: usize) {
    println!("{{");
    println!("  \"files_analyzed\": {},", file_count);
    println!(
        "  \"total_keywords\": {},",
        sorted_counts
            .iter()
            .map(|(_, count)| **count)
            .sum::<usize>()
    );
    println!("  \"keywords\": {{");

    let mut first = true;
    for (keyword, count) in sorted_counts {
        if **count > 0 {
            if !first {
                println!(",");
            }
            print!("    \"{}\": {}", keyword, count);
            first = false;
        }
    }

    if !first {
        println!();
    }
    println!("  }}");
    println!("}}");
}

fn print_csv(sorted_counts: &[(&String, &usize)], file_count: usize) {
    println!("keyword,count");
    println!("_files_analyzed,{}", file_count);
    println!(
        "_total_keywords,{}",
        sorted_counts
            .iter()
            .map(|(_, count)| **count)
            .sum::<usize>()
    );

    for (keyword, count) in sorted_counts {
        if **count > 0 {
            println!("{},{}", keyword, count);
        }
    }
}
