use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const TYPESCRIPT_KEYWORDS: &[&str] = &[
    // JavaScript/TypeScript keywords
    "abstract",
    "any",
    "as",
    "asserts",
    "async",
    "await",
    "boolean",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "constructor",
    "continue",
    "debugger",
    "declare",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "from",
    "function",
    "get",
    "if",
    "implements",
    "import",
    "in",
    "infer",
    "instanceof",
    "interface",
    "is",
    "keyof",
    "let",
    "namespace",
    "never",
    "new",
    "null",
    "number",
    "object",
    "of",
    "override",
    "package",
    "private",
    "protected",
    "public",
    "readonly",
    "require",
    "return",
    "set",
    "static",
    "string",
    "super",
    "switch",
    "symbol",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "undefined",
    "unique",
    "unknown",
    "var",
    "void",
    "while",
    "with",
    "yield",
    // TypeScript-specific types
    "bigint",
    "intrinsic",
    "global",
    "module",
    "satisfies",
    "out",
];

pub fn analyze_directory(
    path: &str,
    total_counts: &mut HashMap<String, usize>,
    file_count: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(path);

    if path.is_file() && is_typescript_file(&path) {
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
            } else if is_typescript_file(&entry_path) {
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
        matches!(name, "target" | ".git" | "node_modules" | "dist" | "build")
    } else {
        false
    }
}

pub fn is_typescript_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext, "ts" | "tsx" | "js" | "jsx")
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
    for word in content.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$') {
        if TYPESCRIPT_KEYWORDS.contains(&word) {
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
        let content = "function main() { let x = 42; if (x > 0) { return; } }";
        let counts = count_keywords(content);

        assert_eq!(counts.get("function"), Some(&1));
        assert_eq!(counts.get("let"), Some(&1));
        assert_eq!(counts.get("if"), Some(&1));
        assert_eq!(counts.get("return"), Some(&1));

        // Test TypeScript-specific keywords
        let content = "interface User { name: string; age: number; } type ID = string | number;";
        let counts = count_keywords(content);
        assert_eq!(counts.get("interface"), Some(&1));
        assert_eq!(counts.get("string"), Some(&2));
        assert_eq!(counts.get("number"), Some(&2));
        assert_eq!(counts.get("type"), Some(&1));

        // Test multiple occurrences
        let content = "let x = 1; let y = 2; let z = x + y;";
        let counts = count_keywords(content);
        assert_eq!(counts.get("let"), Some(&3));

        // Test no keywords
        let content = "hello world 123 test";
        let counts = count_keywords(content);
        assert!(counts.is_empty());
    }

    #[test]
    fn test_is_typescript_file() {
        use std::path::PathBuf;

        // TypeScript files should be detected
        assert!(is_typescript_file(&PathBuf::from("app.ts")));
        assert!(is_typescript_file(&PathBuf::from("component.tsx")));
        assert!(is_typescript_file(&PathBuf::from("script.js")));
        assert!(is_typescript_file(&PathBuf::from("component.jsx")));
        assert!(is_typescript_file(&PathBuf::from("path/to/file.ts")));

        // Non-TypeScript files should not be detected
        assert!(!is_typescript_file(&PathBuf::from("file.rs")));
        assert!(!is_typescript_file(&PathBuf::from("file.py")));
        assert!(!is_typescript_file(&PathBuf::from("file.txt")));
        assert!(!is_typescript_file(&PathBuf::from("file")));
    }

    #[test]
    fn test_should_skip_dir() {
        use std::path::PathBuf;

        // Directories that should be skipped
        assert!(should_skip_dir(&PathBuf::from("node_modules")));
        assert!(should_skip_dir(&PathBuf::from("dist")));
        assert!(should_skip_dir(&PathBuf::from("build")));
        assert!(should_skip_dir(&PathBuf::from("target")));
        assert!(should_skip_dir(&PathBuf::from(".git")));

        // Directories that should not be skipped
        assert!(!should_skip_dir(&PathBuf::from("src")));
        assert!(!should_skip_dir(&PathBuf::from("lib")));
        assert!(!should_skip_dir(&PathBuf::from("components")));
        assert!(!should_skip_dir(&PathBuf::from("utils")));
    }

    #[test]
    fn test_typescript_specific_keywords() {
        // Test TypeScript-only keywords
        let content = "abstract class Base { abstract method(): void; }";
        let counts = count_keywords(content);
        assert_eq!(counts.get("abstract"), Some(&2));
        assert_eq!(counts.get("class"), Some(&1));
        assert_eq!(counts.get("void"), Some(&1));

        // Test generics and advanced types
        let content = "type Result<T> = T | undefined; const x: unknown = null;";
        let counts = count_keywords(content);
        assert_eq!(counts.get("type"), Some(&1));
        assert_eq!(counts.get("undefined"), Some(&1));
        assert_eq!(counts.get("const"), Some(&1));
        assert_eq!(counts.get("unknown"), Some(&1));
        assert_eq!(counts.get("null"), Some(&1));
    }

    #[test]
    fn test_all_typescript_keywords_recognized() {
        // Test that all keywords in TYPESCRIPT_KEYWORDS are properly recognized
        for keyword in TYPESCRIPT_KEYWORDS {
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

    #[test]
    fn test_analyze_file_error_cases() {
        use std::path::PathBuf;
        let mut counts = HashMap::new();

        // Test with non-existent file
        let result = analyze_file(&PathBuf::from("non_existent_file.ts"), &mut counts);
        assert!(result.is_err());

        // Test with directory instead of file (should error)
        let result = analyze_file(&PathBuf::from("/"), &mut counts);
        assert!(result.is_err());
    }

    #[test]
    fn test_keyword_edge_cases() {
        // Test keywords with special characters ($ is allowed in identifiers)
        let content = "let $var = 42; const _private = true;";
        let counts = count_keywords(content);
        assert_eq!(counts.get("let"), Some(&1));
        assert_eq!(counts.get("const"), Some(&1));
        assert_eq!(counts.get("true"), Some(&1));

        // Test keywords in comments and strings (should still be counted due to simple tokenization)
        let content = "// This function returns a string\nconst message = \"function test\";";
        let counts = count_keywords(content);
        assert_eq!(counts.get("function"), Some(&2)); // One in comment, one in string
        assert_eq!(counts.get("string"), Some(&1));
        assert_eq!(counts.get("const"), Some(&1));

        // Test JSX-like syntax
        let content = "const element = <div className=\"test\">Hello</div>;";
        let counts = count_keywords(content);
        assert_eq!(counts.get("const"), Some(&1));
        assert_eq!(counts.get("div"), None); // HTML tags are not TypeScript keywords
    }
}
