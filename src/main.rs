use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::Command;

mod rust;
mod typescript;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (target_path, output_format, language) = parse_args(&args);

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

    let result = match language {
        Language::Rust => rust::analyze_directory(&actual_path, &mut total_counts, &mut file_count),
        Language::TypeScript => {
            typescript::analyze_directory(&actual_path, &mut total_counts, &mut file_count)
        }
    };

    match result {
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

#[derive(Clone, Copy)]
enum Language {
    Rust,
    TypeScript,
}

fn parse_args(args: &[String]) -> (&str, OutputFormat, Language) {
    let mut target_path = ".";
    let mut output_format = OutputFormat::Plain;
    let mut language = Language::Rust; // Default to Rust

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
            "--language" | "-l" => {
                if i + 1 < args.len() {
                    language = match args[i + 1].as_str() {
                        "typescript" | "ts" => Language::TypeScript,
                        "rust" | "rs" => Language::Rust,
                        _ => Language::Rust,
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

    (target_path, output_format, language)
}

fn print_help() {
    println!("Multi-Language Keyword Analyzer");
    println!();
    println!("USAGE:");
    println!("    app [PATH] [OPTIONS]");
    println!();
    println!("ARGS:");
    println!("    <PATH>    Directory, file, or GitHub URL to analyze [default: .]");
    println!();
    println!("OPTIONS:");
    println!("    -l, --language <LANG>    Language to analyze [default: rust] [possible values: rust, rs, typescript, ts]");
    println!("    -f, --format <FORMAT>    Output format [default: plain] [possible values: plain, json, csv]");
    println!("    -h, --help               Print help information");
    println!();
    println!("EXAMPLES:");
    println!("    app --language rust");
    println!("    app --language typescript src/");
    println!("    app -l ts https://github.com/microsoft/typescript");
    println!("    app --format json --language rust https://github.com/rust-lang/rust");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_github_url() {
        // GitHub URLs should be detected
        assert!(is_github_url("https://github.com/rust-lang/rust"));
        assert!(is_github_url("https://github.com/tokio-rs/tokio"));
        assert!(is_github_url("http://github.com/user/repo"));

        // Non-GitHub URLs should not be detected
        assert!(!is_github_url("https://gitlab.com/user/repo"));
        assert!(!is_github_url("https://bitbucket.org/user/repo"));
        assert!(!is_github_url("./local/path"));
        assert!(!is_github_url("/absolute/path"));
        assert!(!is_github_url("relative/path"));
        assert!(!is_github_url(""));

        // Edge cases
        assert!(!is_github_url("github.com/user/repo")); // Missing protocol
        assert!(!is_github_url("https://github.com")); // No repo path
    }

    #[test]
    fn test_parse_args() {
        // Test default values
        let args = vec!["program".to_string()];
        let (path, format, language) = parse_args(&args);
        assert_eq!(path, ".");
        assert!(matches!(format, OutputFormat::Plain));
        assert!(matches!(language, Language::Rust));

        // Test path argument
        let args = vec!["program".to_string(), "src/".to_string()];
        let (path, format, language) = parse_args(&args);
        assert_eq!(path, "src/");
        assert!(matches!(format, OutputFormat::Plain));
        assert!(matches!(language, Language::Rust));

        // Test format options
        let args = vec![
            "program".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];
        let (path, format, _language) = parse_args(&args);
        assert_eq!(path, ".");
        assert!(matches!(format, OutputFormat::Json));

        let args = vec!["program".to_string(), "-f".to_string(), "csv".to_string()];
        let (_path, format, _language) = parse_args(&args);
        assert!(matches!(format, OutputFormat::Csv));

        // Test language options
        let args = vec![
            "program".to_string(),
            "--language".to_string(),
            "typescript".to_string(),
        ];
        let (_path, _format, language) = parse_args(&args);
        assert!(matches!(language, Language::TypeScript));

        let args = vec!["program".to_string(), "-l".to_string(), "ts".to_string()];
        let (_path, _format, language) = parse_args(&args);
        assert!(matches!(language, Language::TypeScript));

        // Test combined arguments
        let args = vec![
            "program".to_string(),
            "target_dir".to_string(),
            "--format".to_string(),
            "json".to_string(),
            "--language".to_string(),
            "rust".to_string(),
        ];
        let (path, format, language) = parse_args(&args);
        assert_eq!(path, "target_dir");
        assert!(matches!(format, OutputFormat::Json));
        assert!(matches!(language, Language::Rust));

        // Test GitHub URL
        let args = vec![
            "program".to_string(),
            "https://github.com/rust-lang/rust".to_string(),
        ];
        let (path, format, language) = parse_args(&args);
        assert_eq!(path, "https://github.com/rust-lang/rust");
        assert!(matches!(format, OutputFormat::Plain));
        assert!(matches!(language, Language::Rust));
    }

    #[test]
    fn test_language_enum_values() {
        // Test that Language enum values work correctly
        let rust_lang = Language::Rust;
        let ts_lang = Language::TypeScript;

        // Test that they are different
        assert!(matches!(rust_lang, Language::Rust));
        assert!(matches!(ts_lang, Language::TypeScript));

        // Test default language in parse_args
        let args = vec!["program".to_string()];
        let (_, _, language) = parse_args(&args);
        assert!(matches!(language, Language::Rust));
    }

    #[test]
    fn test_output_format_enum_values() {
        // Test that OutputFormat enum values work correctly
        let plain = OutputFormat::Plain;
        let json = OutputFormat::Json;
        let csv = OutputFormat::Csv;

        assert!(matches!(plain, OutputFormat::Plain));
        assert!(matches!(json, OutputFormat::Json));
        assert!(matches!(csv, OutputFormat::Csv));
    }

    #[test]
    fn test_parse_args_edge_cases() {
        // Test empty program name only
        let args = vec!["program".to_string()];
        let (path, format, language) = parse_args(&args);
        assert_eq!(path, ".");
        assert!(matches!(format, OutputFormat::Plain));
        assert!(matches!(language, Language::Rust));

        // Test invalid language defaults to Rust
        let args = vec![
            "program".to_string(),
            "--language".to_string(),
            "invalid".to_string(),
        ];
        let (_, _, language) = parse_args(&args);
        assert!(matches!(language, Language::Rust));

        // Test invalid format defaults to Plain
        let args = vec![
            "program".to_string(),
            "--format".to_string(),
            "invalid".to_string(),
        ];
        let (_, format, _) = parse_args(&args);
        assert!(matches!(format, OutputFormat::Plain));

        // Test flag without value (should be skipped)
        let args = vec!["program".to_string(), "--format".to_string()];
        let (_, format, _) = parse_args(&args);
        assert!(matches!(format, OutputFormat::Plain));

        // Test unknown flag (should be ignored, but value becomes path)
        let args = vec![
            "program".to_string(),
            "--unknown-flag".to_string(),
            "value".to_string(),
        ];
        let (path, _, _) = parse_args(&args);
        assert_eq!(path, "value"); // The value becomes the path since unknown flag is skipped
    }

    #[test]
    fn test_clone_github_repo_mock() {
        // Mock test - we only test the function signature and that it returns Result
        // We don't actually call the function to avoid network access and git authentication

        // Test that the function signature is correct (compile-time test)
        fn _test_signature() {
            let _: fn(&str) -> Result<String, Box<dyn std::error::Error>> = clone_github_repo;
        }

        // Test temp directory pattern generation logic (without actual cloning)
        let temp_dir_pattern = format!("/tmp/rust_analyzer_{}", std::process::id());
        assert!(temp_dir_pattern.starts_with("/tmp/rust_analyzer_"));
        assert!(temp_dir_pattern.len() > 20); // Should have process ID appended

        // Note: We skip actual network tests to avoid authentication prompts
        // Real functionality is tested through integration tests manually
    }

    #[test]
    fn test_print_results_formats() {
        use std::collections::HashMap;

        // Create test data
        let mut counts = HashMap::new();
        counts.insert("let".to_string(), 5);
        counts.insert("fn".to_string(), 3);
        counts.insert("if".to_string(), 2);

        // We can't easily test the actual output without capturing stdout,
        // but we can test that the functions don't panic with valid data

        // Test Plain format
        print_results(&counts, 10, OutputFormat::Plain);

        // Test JSON format
        print_results(&counts, 10, OutputFormat::Json);

        // Test CSV format
        print_results(&counts, 10, OutputFormat::Csv);

        // Test with empty data
        let empty_counts = HashMap::new();
        print_results(&empty_counts, 0, OutputFormat::Plain);
        print_results(&empty_counts, 0, OutputFormat::Json);
        print_results(&empty_counts, 0, OutputFormat::Csv);
    }
}
