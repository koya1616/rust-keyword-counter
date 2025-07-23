use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const RUST_KEYWORDS: &[&str] = &[
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

pub fn analyze_directory(
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

pub fn should_skip_dir(path: &Path) -> bool {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        matches!(name, "target" | ".git" | "node_modules")
    } else {
        false
    }
}

pub fn analyze_file(
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

pub fn count_keywords(content: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    // Simple tokenization - split on non-alphanumeric characters
    for word in content.split(|c: char| !c.is_alphanumeric() && c != '_') {
        if RUST_KEYWORDS.contains(&word) {
            *counts.entry(word.to_string()).or_insert(0) += 1;
        }
    }

    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_keywords() {
        // Test basic keyword counting
        let content = "fn main() { let x = 42; if x > 0 { return; } }";
        let counts = count_keywords(content);

        assert_eq!(counts.get("fn"), Some(&1));
        assert_eq!(counts.get("let"), Some(&1));
        assert_eq!(counts.get("if"), Some(&1));
        assert_eq!(counts.get("return"), Some(&1));

        // Test multiple occurrences
        let content = "let x = 1; let y = 2; let z = x + y;";
        let counts = count_keywords(content);
        assert_eq!(counts.get("let"), Some(&3));

        // Test no keywords
        let content = "hello world 123 test";
        let counts = count_keywords(content);
        assert!(counts.is_empty());

        // Test keywords in different contexts
        let content = "struct MyStruct { field: i32 } enum MyEnum { Variant }";
        let counts = count_keywords(content);
        assert_eq!(counts.get("struct"), Some(&1));
        assert_eq!(counts.get("enum"), Some(&1));
        assert_eq!(counts.get("i32"), Some(&1));
    }

    #[test]
    fn test_should_skip_dir() {
        use std::path::PathBuf;

        // Directories that should be skipped
        assert!(should_skip_dir(&PathBuf::from("target")));
        assert!(should_skip_dir(&PathBuf::from(".git")));
        assert!(should_skip_dir(&PathBuf::from("node_modules")));
        assert!(should_skip_dir(&PathBuf::from("/path/to/target")));
        assert!(should_skip_dir(&PathBuf::from("./project/.git")));

        // Directories that should not be skipped
        assert!(!should_skip_dir(&PathBuf::from("src")));
        assert!(!should_skip_dir(&PathBuf::from("examples")));
        assert!(!should_skip_dir(&PathBuf::from("tests")));
        assert!(!should_skip_dir(&PathBuf::from("my_target_dir")));
        assert!(!should_skip_dir(&PathBuf::from(".github")));
    }

    #[test]
    fn test_keyword_edge_cases() {
        // Test keywords with underscores
        let content = "let my_var = 42; fn my_function() {}";
        let counts = count_keywords(content);
        assert_eq!(counts.get("let"), Some(&1));
        assert_eq!(counts.get("fn"), Some(&1));
        // my_var and my_function should not be counted as keywords
        assert!(counts.get("my_var").is_none());
        assert!(counts.get("my_function").is_none());

        // Test keywords in comments (should still be counted due to simple tokenization)
        let content = "// This is a fn comment with let and if";
        let counts = count_keywords(content);
        assert_eq!(counts.get("fn"), Some(&1));
        assert_eq!(counts.get("let"), Some(&1));
        assert_eq!(counts.get("if"), Some(&1));

        // Test keywords in strings (should still be counted due to simple tokenization)
        let content = r#"let message = "This contains fn keyword";"#;
        let counts = count_keywords(content);
        assert_eq!(counts.get("let"), Some(&1));
        assert_eq!(counts.get("fn"), Some(&1));
    }

    #[test]
    fn test_all_rust_keywords_recognized() {
        // Test that all keywords in RUST_KEYWORDS are properly recognized
        for keyword in RUST_KEYWORDS {
            let content = format!("{} ", keyword);
            let counts = count_keywords(&content);
            assert_eq!(
                counts.get(*keyword),
                Some(&1),
                "Keyword '{}' was not properly counted",
                keyword
            );
        }
    }
}
