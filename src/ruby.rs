use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const RUBY_KEYWORDS: &[&str] = &[
    // Core Ruby keywords
    "alias",
    "and",
    "begin",
    "break",
    "case",
    "class",
    "def",
    "defined?",
    "do",
    "else",
    "elsif",
    "end",
    "ensure",
    "false",
    "for",
    "if",
    "in",
    "module",
    "next",
    "nil",
    "not",
    "or",
    "redo",
    "rescue",
    "retry",
    "return",
    "self",
    "super",
    "then",
    "true",
    "undef",
    "unless",
    "until",
    "when",
    "while",
    "yield",
    // Special variables and constants
    "__FILE__",
    "__LINE__",
    "__ENCODING__",
    // Access modifiers
    "private",
    "protected",
    "public",
    // Block keywords
    "proc",
    "lambda",
    // Exception handling
    "raise",
    "throw",
    "catch",
    // Iteration and enumeration
    "each",
    "map",
    "select",
    "reject",
    "find",
    "collect",
    "inject",
    "reduce",
    // String and regex
    "gsub",
    "sub",
    "match",
    // Metaprogramming
    "attr_reader",
    "attr_writer",
    "attr_accessor",
    "include",
    "extend",
    "prepend",
    // Class and module methods
    "initialize",
    "new",
    "allocate",
    "freeze",
    "dup",
    "clone",
    // Comparison and logical
    "eql?",
    "equal?",
    "respond_to?",
    "kind_of?",
    "instance_of?",
    "is_a?",
];

pub fn analyze_directory(
    path: &str,
    total_counts: &mut HashMap<String, usize>,
    file_count: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(path);

    if path.is_file() && is_ruby_file(&path) {
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
            } else if is_ruby_file(&entry_path) {
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
        matches!(name, "target" | ".git" | "node_modules" | "vendor" | "tmp" | "log" | ".bundle")
    } else {
        false
    }
}

pub fn is_ruby_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        matches!(ext, "rb" | "rake" | "gemspec")
    } else if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        matches!(filename, "Rakefile" | "Gemfile" | "Guardfile" | "Capfile" | "Vagrantfile")
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

    // Simple tokenization - split on non-alphanumeric characters, but preserve ? and _
    for word in content.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '?') {
        if RUBY_KEYWORDS.contains(&word) {
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
        let content = "def hello_world\n  if true\n    puts 'Hello, World!'\n  end\nend";
        let counts = count_keywords(content);

        assert_eq!(counts.get("def"), Some(&1));
        assert_eq!(counts.get("if"), Some(&1));
        assert_eq!(counts.get("true"), Some(&1));
        assert_eq!(counts.get("end"), Some(&2));

        // Test class and module keywords
        let content = "class MyClass < SuperClass\n  include MyModule\n  def initialize\n    super\n  end\nend";
        let counts = count_keywords(content);
        assert_eq!(counts.get("class"), Some(&1));
        assert_eq!(counts.get("include"), Some(&1));
        assert_eq!(counts.get("def"), Some(&1));
        assert_eq!(counts.get("initialize"), Some(&1));
        assert_eq!(counts.get("super"), Some(&1));
        assert_eq!(counts.get("end"), Some(&2));

        // Test multiple occurrences
        let content = "def method1\nend\ndef method2\nend\ndef method3\nend";
        let counts = count_keywords(content);
        assert_eq!(counts.get("def"), Some(&3));
        assert_eq!(counts.get("end"), Some(&3));

        // Test no keywords
        let content = "hello world 123 test";
        let counts = count_keywords(content);
        assert!(counts.is_empty());
    }

    #[test]
    fn test_is_ruby_file() {
        use std::path::PathBuf;

        // Ruby files should be detected
        assert!(is_ruby_file(&PathBuf::from("app.rb")));
        assert!(is_ruby_file(&PathBuf::from("config.rb")));
        assert!(is_ruby_file(&PathBuf::from("script.rake")));
        assert!(is_ruby_file(&PathBuf::from("my_gem.gemspec")));
        assert!(is_ruby_file(&PathBuf::from("Rakefile")));
        assert!(is_ruby_file(&PathBuf::from("Gemfile")));
        assert!(is_ruby_file(&PathBuf::from("Guardfile")));
        assert!(is_ruby_file(&PathBuf::from("Capfile")));
        assert!(is_ruby_file(&PathBuf::from("Vagrantfile")));
        assert!(is_ruby_file(&PathBuf::from("path/to/file.rb")));

        // Non-Ruby files should not be detected
        assert!(!is_ruby_file(&PathBuf::from("file.rs")));
        assert!(!is_ruby_file(&PathBuf::from("file.py")));
        assert!(!is_ruby_file(&PathBuf::from("file.js")));
        assert!(!is_ruby_file(&PathBuf::from("file.txt")));
        assert!(!is_ruby_file(&PathBuf::from("file")));
        assert!(!is_ruby_file(&PathBuf::from("app.rb.bak")));
    }

    #[test]
    fn test_should_skip_dir() {
        use std::path::PathBuf;

        // Directories that should be skipped
        assert!(should_skip_dir(&PathBuf::from("vendor")));
        assert!(should_skip_dir(&PathBuf::from("tmp")));
        assert!(should_skip_dir(&PathBuf::from("log")));
        assert!(should_skip_dir(&PathBuf::from(".bundle")));
        assert!(should_skip_dir(&PathBuf::from("target")));
        assert!(should_skip_dir(&PathBuf::from(".git")));
        assert!(should_skip_dir(&PathBuf::from("node_modules")));

        // Directories that should not be skipped
        assert!(!should_skip_dir(&PathBuf::from("lib")));
        assert!(!should_skip_dir(&PathBuf::from("app")));
        assert!(!should_skip_dir(&PathBuf::from("config")));
        assert!(!should_skip_dir(&PathBuf::from("spec")));
        assert!(!should_skip_dir(&PathBuf::from("test")));
    }

    #[test]
    fn test_ruby_specific_keywords() {
        // Test Ruby-specific keywords and methods
        let content = "attr_accessor :name, :age\nattr_reader :id\nattr_writer :email";
        let counts = count_keywords(content);
        assert_eq!(counts.get("attr_accessor"), Some(&1));
        assert_eq!(counts.get("attr_reader"), Some(&1));
        assert_eq!(counts.get("attr_writer"), Some(&1));

        // Test metaprogramming keywords
        let content = "module MyModule\n  extend ActiveSupport::Concern\n  prepend SomeModule\nend";
        let counts = count_keywords(content);
        assert_eq!(counts.get("module"), Some(&1));
        assert_eq!(counts.get("extend"), Some(&1));
        assert_eq!(counts.get("prepend"), Some(&1));
        assert_eq!(counts.get("end"), Some(&1));

        // Test special variables
        let content = "puts __FILE__\nputs __LINE__\nputs __ENCODING__";
        let counts = count_keywords(content);
        assert_eq!(counts.get("__FILE__"), Some(&1));
        assert_eq!(counts.get("__LINE__"), Some(&1));
        assert_eq!(counts.get("__ENCODING__"), Some(&1));
    }

    #[test]
    fn test_ruby_blocks_and_iteration() {
        // Test block and iteration keywords
        let content = "[1,2,3].each { |x| puts x }\n[1,2,3].map(&:to_s).select { |s| s.length > 0 }";
        let counts = count_keywords(content);
        assert_eq!(counts.get("each"), Some(&1));
        assert_eq!(counts.get("map"), Some(&1));
        assert_eq!(counts.get("select"), Some(&1));

        // Test proc and lambda
        let content = "my_proc = proc { |x| x * 2 }\nmy_lambda = lambda { |x| x + 1 }";
        let counts = count_keywords(content);
        assert_eq!(counts.get("proc"), Some(&1));
        assert_eq!(counts.get("lambda"), Some(&1));
    }

    #[test]
    fn test_ruby_question_mark_methods() {
        // Test methods with question marks
        let content = "obj.respond_to?(:method_name)\nobj.nil?\nobj.is_a?(String)";
        let counts = count_keywords(content);
        assert_eq!(counts.get("respond_to?"), Some(&1));
        assert_eq!(counts.get("is_a?"), Some(&1));
        // Note: nil? is not in our keyword list as it's a method, not a keyword
    }

    #[test]
    fn test_all_ruby_keywords_recognized() {
        // Test that all keywords in RUBY_KEYWORDS are properly recognized
        for keyword in RUBY_KEYWORDS {
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
        let result = analyze_file(&PathBuf::from("non_existent_file.rb"), &mut counts);
        assert!(result.is_err());

        // Test with directory instead of file (should error)
        let result = analyze_file(&PathBuf::from("/"), &mut counts);
        assert!(result.is_err());
    }

    #[test]
    fn test_keyword_edge_cases() {
        // Test keywords in comments and strings (should still be counted due to simple tokenization)
        let content = "# This def method returns true\nputs \"The class keyword\"";
        let counts = count_keywords(content);
        assert_eq!(counts.get("def"), Some(&1));
        assert_eq!(counts.get("true"), Some(&1));
        assert_eq!(counts.get("class"), Some(&1));

        // Test keywords with underscores and special characters
        let content = "def my_method\n  return false unless defined?(something)\nend";
        let counts = count_keywords(content);
        assert_eq!(counts.get("def"), Some(&1));
        assert_eq!(counts.get("return"), Some(&1));
        assert_eq!(counts.get("false"), Some(&1));
        assert_eq!(counts.get("unless"), Some(&1));
        assert_eq!(counts.get("defined?"), Some(&1));
        assert_eq!(counts.get("end"), Some(&1));
    }

    #[test]
    fn test_exception_handling() {
        // Test exception handling keywords
        let content = "begin\n  raise StandardError\nrescue => e\n  retry\nensure\n  puts 'cleanup'\nend";
        let counts = count_keywords(content);
        assert_eq!(counts.get("begin"), Some(&1));
        assert_eq!(counts.get("raise"), Some(&1));
        assert_eq!(counts.get("rescue"), Some(&1));
        assert_eq!(counts.get("retry"), Some(&1));
        assert_eq!(counts.get("ensure"), Some(&1));
        assert_eq!(counts.get("end"), Some(&1));
    }
}