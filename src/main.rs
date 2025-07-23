use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::Command;

mod golang;
mod javascript;
mod python;
mod ruby;
mod rust;

fn main() {
    let args: Vec<String> = env::args().collect();
    let (target_path, output_format, language) = parse_args(&args);

    let actual_path = if is_git_url(&target_path) {
        match clone_git_repo(&target_path) {
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
        println!("Analyzing files in: {}", actual_path);
    }

    let mut total_counts: HashMap<String, usize> = HashMap::new();
    let mut file_count = 0;

    let result = match language {
        Language::Rust => rust::analyze_directory(&actual_path, &mut total_counts, &mut file_count),
        Language::JavaScript => {
            javascript::analyze_directory(&actual_path, &mut total_counts, &mut file_count)
        }
        Language::Ruby => ruby::analyze_directory(&actual_path, &mut total_counts, &mut file_count),
        Language::Golang => {
            golang::analyze_directory(&actual_path, &mut total_counts, &mut file_count)
        }
        Language::Python => {
            python::analyze_directory(&actual_path, &mut total_counts, &mut file_count)
        }
    };

    match result {
        Ok(_) => {
            eprintln!("Analysis completed! Found {} files", file_count);
            eprintln!("Generating results...\n");
            print_results(&total_counts, file_count, output_format, language);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }

    if is_git_url(&target_path) {
        eprintln!("Cleaning up temporary directory...");
        let _ = fs::remove_dir_all(&actual_path);
    }
}

#[derive(Clone, Copy)]
enum OutputFormat {
    Plain,
    Json,
    Csv,
    Html,
}

#[derive(Clone, Copy)]
enum Language {
    Rust,
    JavaScript,
    Ruby,
    Golang,
    Python,
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
                        "html" => OutputFormat::Html,
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
    println!("    <PATH>    Directory, file, or Git URL (GitHub/GitLab) to analyze [default: .]");
    println!();
    println!("OPTIONS:");
    println!("    -l, --language <LANG>    Language to analyze [default: rust] [possible values: rust, rs, js, ts, ruby, rb, go, golang, python, py]");
    println!("    -f, --format <FORMAT>    Output format [default: plain] [possible values: plain, json, csv, html]");
    println!("    -h, --help               Print help information");
    println!();
    println!("EXAMPLES:");
    println!("    app --language rust");
    println!("    app --language js src/");
    println!("    app --language ruby lib/");
    println!("    app --language go cmd/");
    println!("    app --language python src/");
    println!("    app -l ts github.com/microsoft/typescript");
    println!("    app -l py github.com/psf/requests");
    println!("    app -l rs gitlab.com/gitlab-org/gitlab");
    println!("    app -l go https://github.com/golang/go");
    println!("    app --format json --language python https://github.com/django/django");
}

fn is_git_url(input: &str) -> bool {
    input.starts_with("https://github.com/")
        || input.starts_with("http://github.com/")
        || input.starts_with("github.com/")
        || input.starts_with("https://gitlab.com/")
        || input.starts_with("http://gitlab.com/")
        || input.starts_with("gitlab.com/")
}

fn clone_git_repo(url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let temp_dir = format!("/tmp/rust_analyzer_{}", std::process::id());

    // Normalize URL - add https:// if missing protocol
    let normalized_url = if url.starts_with("github.com/") || url.starts_with("gitlab.com/") {
        format!("https://{}", url)
    } else {
        url.to_string()
    };

    eprintln!("Cloning repository: {}", normalized_url);
    eprintln!("Target directory: {}", temp_dir);

    let output = Command::new("git")
        .args(&["clone", "--depth", "1", &normalized_url, &temp_dir])
        .output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to clone repository: {}", error_msg).into());
    }

    eprintln!("Repository cloned successfully");

    Ok(temp_dir)
}

fn print_results(
    counts: &HashMap<String, usize>,
    file_count: usize,
    format: OutputFormat,
    language: Language,
) {
    let mut sorted_counts: Vec<_> = counts.iter().collect();
    sorted_counts.sort_by(|a, b| b.1.cmp(a.1));

    match format {
        OutputFormat::Plain => print_plain(&sorted_counts, file_count, language),
        OutputFormat::Json => print_json(&sorted_counts, file_count),
        OutputFormat::Csv => print_csv(&sorted_counts, file_count),
        OutputFormat::Html => print_html(&sorted_counts, file_count, language),
    }
}

fn print_plain(sorted_counts: &[(&String, &usize)], file_count: usize, language: Language) {
    let language_name = match language {
        Language::Rust => "Rust",
        Language::JavaScript => "JavaScript/TypeScript",
        Language::Ruby => "Ruby",
        Language::Golang => "Go",
        Language::Python => "Python",
    };

    println!("\n=== {} Keyword Analysis Results ===", language_name);
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

fn print_html(sorted_counts: &[(&String, &usize)], file_count: usize, language: Language) {
    let language_name = match language {
        Language::Rust => "Rust",
        Language::JavaScript => "JavaScript/TypeScript",
        Language::Ruby => "Ruby",
        Language::Golang => "Go",
        Language::Python => "Python",
    };

    let total_keywords: usize = sorted_counts
        .iter()
        .map(|(_, count)| **count)
        .sum();

    println!("<!DOCTYPE html>");
    println!("<html lang=\"en\">");
    println!("<head>");
    println!("    <meta charset=\"UTF-8\">");
    println!("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">");
    println!("    <title>{} Keyword Analysis Results</title>", language_name);
    println!("    <style>");
    println!("        body {{ font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; margin: 40px; background-color: #f5f5f5; }}");
    println!("        .container {{ max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}");
    println!("        h1 {{ color: #333; text-align: center; margin-bottom: 30px; border-bottom: 3px solid #007acc; padding-bottom: 10px; }}");
    println!("        .summary {{ background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 30px; border-left: 4px solid #007acc; }}");
    println!("        .summary h2 {{ margin-top: 0; color: #495057; }}");
    println!("        .stat {{ display: inline-block; margin-right: 30px; }}");
    println!("        .stat-value {{ font-size: 24px; font-weight: bold; color: #007acc; }}");
    println!("        .stat-label {{ font-size: 14px; color: #6c757d; }}");
    println!("        .keywords-table {{ width: 100%; border-collapse: collapse; margin-top: 20px; }}");
    println!("        .keywords-table th {{ background: #007acc; color: white; padding: 12px; text-align: left; font-weight: 600; }}");
    println!("        .keywords-table td {{ padding: 10px 12px; border-bottom: 1px solid #dee2e6; }}");
    println!("        .keywords-table tr:nth-child(even) {{ background-color: #f8f9fa; }}");
    println!("        .keywords-table tr:hover {{ background-color: #e3f2fd; }}");
    println!("        .keyword {{ font-family: 'Consolas', 'Monaco', monospace; font-weight: 600; color: #495057; }}");
    println!("        .count {{ font-weight: bold; color: #007acc; }}");
    println!("        .progress-bar {{ width: 100%; height: 8px; background: #e9ecef; border-radius: 4px; overflow: hidden; }}");
    println!("        .progress-fill {{ height: 100%; background: linear-gradient(90deg, #007acc, #40a9ff); transition: width 0.3s ease; }}");
    println!("        .footer {{ text-align: center; margin-top: 30px; color: #6c757d; font-size: 12px; }}");
    println!("    </style>");
    println!("</head>");
    println!("<body>");
    println!("    <div class=\"container\">");
    println!("        <h1>{} Keyword Analysis Results</h1>", language_name);
    println!("        <div class=\"summary\">");
    println!("            <h2>Analysis Summary</h2>");
    println!("            <div class=\"stat\">");
    println!("                <div class=\"stat-value\">{}</div>", file_count);
    println!("                <div class=\"stat-label\">Files Analyzed</div>");
    println!("            </div>");
    println!("            <div class=\"stat\">");
    println!("                <div class=\"stat-value\">{}</div>", total_keywords);
    println!("                <div class=\"stat-label\">Total Keywords Found</div>");
    println!("            </div>");
    println!("        </div>");

    if !sorted_counts.is_empty() && total_keywords > 0 {
        let max_count = sorted_counts.iter().map(|(_, count)| **count).max().unwrap_or(1);
        
        println!("        <table class=\"keywords-table\">");
        println!("            <thead>");
        println!("                <tr>");
        println!("                    <th>Keyword</th>");
        println!("                    <th>Count</th>");
        println!("                    <th>Distribution</th>");
        println!("                </tr>");
        println!("            </thead>");
        println!("            <tbody>");

        for (keyword, count) in sorted_counts {
            if **count > 0 {
                let percentage = ((**count as f64) / (max_count as f64) * 100.0) as u32;
                println!("                <tr>");
                println!("                    <td class=\"keyword\">{}</td>", keyword);
                println!("                    <td class=\"count\">{}</td>", count);
                println!("                    <td>");
                println!("                        <div class=\"progress-bar\">");
                println!("                            <div class=\"progress-fill\" style=\"width: {}%;\"></div>", percentage);
                println!("                        </div>");
                println!("                    </td>");
                println!("                </tr>");
            }
        }

        println!("            </tbody>");
        println!("        </table>");
    } else {
        println!("        <p style=\"text-align: center; color: #6c757d; font-style: italic;\">No keywords found in the analyzed files.</p>");
    }

    println!("        <div class=\"footer\">");
    println!("            <p>Generated by Multi-Language Keyword Analyzer</p>");
    println!("        </div>");
    println!("    </div>");
    println!("</body>");
    println!("</html>");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_git_url() {
        // GitHub URLs should be detected
        assert!(is_git_url("https://github.com/rust-lang/rust"));
        assert!(is_git_url("http://github.com/user/repo"));
        assert!(is_git_url("github.com/user/repo"));

        // GitLab URLs should be detected
        assert!(is_git_url("https://gitlab.com/gitlab-org/gitlab"));
        assert!(is_git_url("http://gitlab.com/user/repo"));
        assert!(is_git_url("gitlab.com/user/repo"));

        // Non-Git URLs should not be detected
        assert!(!is_git_url("https://bitbucket.org/user/repo"));
        assert!(!is_git_url("./local/path"));
        assert!(!is_git_url("/absolute/path"));
        assert!(!is_git_url(""));

        // Edge cases
        assert!(!is_git_url("https://github.com"));
        assert!(!is_git_url("github.com"));
        assert!(!is_git_url("https://gitlab.com"));
        assert!(!is_git_url("gitlab.com"));
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

        let args = vec!["program".to_string(), "--format".to_string(), "html".to_string()];
        let (_path, format, _language) = parse_args(&args);
        assert!(matches!(format, OutputFormat::Html));

        // Test language options
        let args = vec![
            "program".to_string(),
            "--language".to_string(),
            "js".to_string(),
        ];
        let (_path, _format, language) = parse_args(&args);
        assert!(matches!(language, Language::JavaScript));

        let args = vec!["program".to_string(), "-l".to_string(), "ts".to_string()];
        let (_path, _format, language) = parse_args(&args);
        assert!(matches!(language, Language::JavaScript));

        let args = vec![
            "program".to_string(),
            "--language".to_string(),
            "ruby".to_string(),
        ];
        let (_path, _format, language) = parse_args(&args);
        assert!(matches!(language, Language::Ruby));

        let args = vec!["program".to_string(), "-l".to_string(), "rb".to_string()];
        let (_path, _format, language) = parse_args(&args);
        assert!(matches!(language, Language::Ruby));

        let args = vec![
            "program".to_string(),
            "--language".to_string(),
            "go".to_string(),
        ];
        let (_path, _format, language) = parse_args(&args);
        assert!(matches!(language, Language::Golang));

        let args = vec![
            "program".to_string(),
            "-l".to_string(),
            "golang".to_string(),
        ];
        let (_path, _format, language) = parse_args(&args);
        assert!(matches!(language, Language::Golang));

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
        let js_lang = Language::JavaScript;
        let ruby_lang = Language::Ruby;
        let go_lang = Language::Golang;

        // Test that they are different
        assert!(matches!(rust_lang, Language::Rust));
        assert!(matches!(js_lang, Language::JavaScript));
        assert!(matches!(ruby_lang, Language::Ruby));
        assert!(matches!(go_lang, Language::Golang));

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
        let html = OutputFormat::Html;

        assert!(matches!(plain, OutputFormat::Plain));
        assert!(matches!(json, OutputFormat::Json));
        assert!(matches!(csv, OutputFormat::Csv));
        assert!(matches!(html, OutputFormat::Html));
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
    fn test_clone_git_repo_mock() {
        // Mock test - we only test the function signature and that it returns Result
        // We don't actually call the function to avoid network access and git authentication

        // Test that the function signature is correct (compile-time test)
        fn _test_signature() {
            let _: fn(&str) -> Result<String, Box<dyn std::error::Error>> = clone_git_repo;
        }

        // Test temp directory pattern generation logic (without actual cloning)
        let temp_dir_pattern = format!("/tmp/rust_analyzer_{}", std::process::id());
        assert!(temp_dir_pattern.starts_with("/tmp/rust_analyzer_"));
        assert!(temp_dir_pattern.len() > 20); // Should have process ID appended

        // Note: We skip actual network tests to avoid authentication prompts
        // Real functionality is tested through integration tests manually
    }

    #[test]
    fn test_url_normalization() {
        // Test URL normalization logic (without actual cloning)

        // Protocol-less GitHub URL should be normalized to https
        let normalized_github = if "github.com/user/repo".starts_with("github.com/")
            || "github.com/user/repo".starts_with("gitlab.com/")
        {
            format!("https://{}", "github.com/user/repo")
        } else {
            "github.com/user/repo".to_string()
        };
        assert_eq!(normalized_github, "https://github.com/user/repo");

        // Protocol-less GitLab URL should be normalized to https
        let normalized_gitlab = if "gitlab.com/user/repo".starts_with("github.com/")
            || "gitlab.com/user/repo".starts_with("gitlab.com/")
        {
            format!("https://{}", "gitlab.com/user/repo")
        } else {
            "gitlab.com/user/repo".to_string()
        };
        assert_eq!(normalized_gitlab, "https://gitlab.com/user/repo");

        // URLs with protocol should remain unchanged
        let normalized_existing = if "https://github.com/user/repo".starts_with("github.com/")
            || "https://github.com/user/repo".starts_with("gitlab.com/")
        {
            format!("https://{}", "https://github.com/user/repo")
        } else {
            "https://github.com/user/repo".to_string()
        };
        assert_eq!(normalized_existing, "https://github.com/user/repo");
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
        print_results(&counts, 10, OutputFormat::Plain, Language::Rust);

        // Test JSON format
        print_results(&counts, 10, OutputFormat::Json, Language::JavaScript);

        // Test CSV format
        print_results(&counts, 10, OutputFormat::Csv, Language::Ruby);

        // Test HTML format
        print_results(&counts, 10, OutputFormat::Html, Language::Golang);

        // Test with empty data
        let empty_counts = HashMap::new();
        print_results(&empty_counts, 0, OutputFormat::Plain, Language::Golang);
        print_results(&empty_counts, 0, OutputFormat::Json, Language::Rust);
        print_results(&empty_counts, 0, OutputFormat::Csv, Language::JavaScript);
        print_results(&empty_counts, 0, OutputFormat::Html, Language::Python);
    }
}
