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
    matches!(
      name,
      "target" | ".git" | "node_modules" | "vendor" | "tmp" | "log" | ".bundle"
    )
  } else {
    false
  }
}

pub fn is_ruby_file(path: &Path) -> bool {
  if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
    matches!(ext, "rb" | "rake" | "gemspec")
  } else if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
    matches!(
      filename,
      "Rakefile" | "Gemfile" | "Guardfile" | "Capfile" | "Vagrantfile"
    )
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
  let mut chars = content.chars().peekable();
  let mut current_token = String::new();
  let mut in_string = false;
  let mut in_comment = false;
  let mut string_char = '\0';

  while let Some(c) = chars.next() {
    match c {
      // Handle single-line comments (Ruby uses #)
      '#' if !in_string => {
        in_comment = true;
        if !current_token.is_empty() {
          check_and_count_token(&current_token, &mut counts);
          current_token.clear();
        }
        continue;
      }
      // End single-line comment
      '\n' if in_comment => {
        in_comment = false;
        continue;
      }
      // Handle strings (double quotes and single quotes)
      '"' | '\'' if !in_comment => {
        if !in_string {
          in_string = true;
          string_char = c;
          if !current_token.is_empty() {
            check_and_count_token(&current_token, &mut counts);
            current_token.clear();
          }
        } else if c == string_char {
          in_string = false;
          string_char = '\0';
        }
        continue;
      }
      // Handle escape sequences in strings
      '\\' if in_string => {
        // Skip the next character if we're in a string (escape sequence)
        chars.next();
        continue;
      }
      // Skip content inside strings and comments
      _ if in_string || in_comment => {
        continue;
      }
      // Handle regular tokens (Ruby allows ? in method names)
      _ => {
        if c.is_alphanumeric() || c == '_' || c == '?' {
          current_token.push(c);
        } else {
          if !current_token.is_empty() {
            check_and_count_token(&current_token, &mut counts);
            current_token.clear();
          }
        }
      }
    }
  }

  // Check final token
  if !current_token.is_empty() {
    check_and_count_token(&current_token, &mut counts);
  }

  counts
}

fn check_and_count_token(token: &str, counts: &mut HashMap<String, usize>) {
  if RUBY_KEYWORDS.contains(&token) {
    *counts.entry(token.to_string()).or_insert(0) += 1;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_count_keywords() {
    // Core Ruby Keywords and Control Flow
    let core_keywords = r#"
            def main_method
                if condition
                    return true
                elsif other_condition
                    return false
                else
                    return nil
                end
                
                unless negative_condition
                    do_something
                end
                
                case value
                when 1
                    puts "one"
                when 2
                    puts "two"
                else
                    puts "other"
                end
                
                while condition
                    break if should_exit
                    next if should_skip
                    redo if should_retry
                end
                
                until condition
                    # loop until condition is true
                end
                
                for item in collection
                    # iterate through collection
                end
            end
        "#;

    // Class and Module Definitions
    let class_module_definitions = r#"
            class Person
                def initialize(name)
                    @name = name
                end
                
                def name
                    @name
                end
                
                private
                
                def private_method
                    # private implementation
                end
                
                protected
                
                def protected_method
                    # protected implementation
                end
                
                public
                
                def public_method
                    # public interface
                end
            end
            
            module Enumerable
                def each
                    # implementation
                end
            end
        "#;

    // Metaprogramming and Attributes
    let metaprogramming = r#"
            class User
                attr_reader :id
                attr_writer :password
                attr_accessor :name, :email
                
                include Comparable
                extend ClassMethods
                prepend SecurityModule
                
                alias old_name name
                undef dangerous_method
            end
        "#;

    // Exception Handling
    let exception_handling = r#"
            def risky_operation
                begin
                    # risky code here
                    raise StandardError, "Something went wrong" if error_condition
                rescue StandardError => e
                    retry if should_retry
                    # handle error
                ensure
                    # cleanup code
                end
                
                catch(:early_exit) do
                    throw(:early_exit) if should_exit_early
                    # normal processing
                end
            end
        "#;

    // Blocks, Procs and Lambdas
    let blocks_procs = r#"
            def demonstrate_blocks
                # Block with yield
                yield if block_given?
                
                # Proc creation
                my_proc = proc { |x| x * 2 }
                
                # Lambda creation
                my_lambda = lambda { |x| x + 1 }
                
                # Block methods
                numbers = [1, 2, 3, 4, 5]
                numbers.each { |n| puts n }
                mapped = numbers.map { |n| n * 2 }
                selected = numbers.select { |n| n.even? }
                rejected = numbers.reject { |n| n.odd? }
                found = numbers.find { |n| n > 3 }
                collected = numbers.collect { |n| n.to_s }
                sum = numbers.inject(0) { |acc, n| acc + n }
                reduced = numbers.reduce(:+)
            end
        "#;

    // String and Pattern Matching
    let string_patterns = r#"
            def string_operations
                text = "Hello, World!"
                
                # String manipulation
                text.gsub(/World/, "Ruby")
                text.sub(/Hello/, "Hi")
                
                # Pattern matching
                if text.match(/Hello/)
                    puts "Found greeting"
                end
            end
        "#;

    // Object Methods and Comparisons
    let object_methods = r#"
            def object_comparisons
                obj = Object.new
                other = Object.allocate
                
                # Object creation and manipulation
                copy = obj.dup
                clone_obj = obj.clone
                obj.freeze
                
                # Type checking and comparisons
                obj.respond_to?(:method_name)
                obj.kind_of?(Object)
                obj.instance_of?(Object)
                obj.is_a?(Object)
                obj.eql?(other)
                obj.equal?(other)
            end
        "#;

    // Special Variables and Constants
    let special_variables = r#"
            def file_info
                current_file = __FILE__
                current_line = __LINE__
                encoding = __ENCODING__
            end
        "#;

    // Logical Operators
    let logical_operators = r#"
            def logical_operations
                result = condition and other_condition
                result = condition or fallback_condition
                result = not negative_condition
                
                # Alternative syntax
                result = condition && other_condition
                result = condition || fallback_condition
                result = !negative_condition
            end
        "#;

    // Special Keywords
    let special_keywords = r#"
            def special_features
                # Check if method or variable is defined
                if defined?(some_variable)
                    puts "Variable exists"
                end
                
                # Reference to current object
                puts self.class
                
                # Call parent method
                super
                
                # Conditional execution
                puts "debug" if debug_mode
                puts "production" unless development_mode
                
                # Then keyword (optional)
                if condition then
                    do_something
                end
            end
        "#;

    // Combine all sections
    let comprehensive_content = format!(
      "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
      core_keywords,
      class_module_definitions,
      metaprogramming,
      exception_handling,
      blocks_procs,
      string_patterns,
      object_methods,
      special_variables,
      logical_operators,
      special_keywords
    );

    let counts = count_keywords(&comprehensive_content);

    // Verify that ALL keywords are present in the comprehensive example
    let mut missing_keywords = Vec::new();
    for &keyword in RUBY_KEYWORDS {
      if counts.get(keyword).is_none() {
        missing_keywords.push(keyword);
      }
    }

    if !missing_keywords.is_empty() {
      panic!(
        "The following keywords are missing from the comprehensive test: {:?}",
        missing_keywords
      );
    }

    // Verify specific keywords appear at least once
    for &keyword in RUBY_KEYWORDS {
      assert!(
        counts.get(keyword).unwrap_or(&0) >= &1,
        "Keyword '{}' should appear at least once in the comprehensive example",
        keyword
      );
    }

    println!(
      "✅ All {} Ruby keywords are properly tested!",
      RUBY_KEYWORDS.len()
    );

    // Test edge case: no keywords
    let no_keywords_content = "hello world 123 test";
    let no_keywords_counts = count_keywords(no_keywords_content);
    assert!(no_keywords_counts.is_empty());
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
    // Test keywords in comments and strings (should NOT be counted with new implementation)
    let content = "# This def method returns true\nputs \"The class keyword\"";
    let counts = count_keywords(content);
    assert_eq!(counts.get("def"), None); // Not counted in comment
    assert_eq!(counts.get("true"), None); // Not counted in comment
    assert_eq!(counts.get("class"), None); // Not counted in string

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
  fn test_string_and_comment_exclusion() {
    // Test single-line comments
    let content = "def test_method\n  # This def should not be counted\nend";
    let counts = count_keywords(content);
    assert_eq!(counts.get("def"), Some(&1)); // Only the actual def
    assert_eq!(counts.get("end"), Some(&1)); // Only the actual end

    // Test various string types
    let content = r#"
      def method_with_strings
        single_quote = 'This has def and class in it'
        double_quote = "This contains return and if keywords"
        return true
      end
    "#;
    let counts = count_keywords(content);
    assert_eq!(counts.get("def"), Some(&1)); // One def declaration
    assert_eq!(counts.get("return"), Some(&1)); // One return statement
    assert_eq!(counts.get("true"), Some(&1)); // One true literal
    assert_eq!(counts.get("end"), Some(&1)); // One end statement
    // These should NOT be counted as they're in strings
    assert_eq!(counts.get("class"), None); // Only in string
    assert_eq!(counts.get("if"), None); // Only in string

    // Test escape sequences in strings
    let content = r#"message = "Quote: \"def test; end\""; def actual_method; end"#;
    let counts = count_keywords(content);
    assert_eq!(counts.get("def"), Some(&1)); // Only the actual def declaration
    assert_eq!(counts.get("end"), Some(&1)); // Only the actual end statement
  }

  #[test]
  fn test_partial_word_matches() {
    // Test that keywords within identifiers are NOT counted
    let content = "def define_method; return return_value; end";
    let counts = count_keywords(content);
    assert_eq!(counts.get("def"), Some(&1)); // Only the actual def keyword
    assert_eq!(counts.get("return"), Some(&1)); // Only the actual return keyword
    assert_eq!(counts.get("end"), Some(&1)); // Only the actual end keyword
    
    // These should NOT be counted as they are part of identifiers
    assert_eq!(counts.get("define_method"), None);
    assert_eq!(counts.get("return_value"), None);
    
    // Test with question mark methods
    let content = "def test?; if respond_to?(:method); return true; end; end";
    let counts = count_keywords(content);
    assert_eq!(counts.get("def"), Some(&1));
    assert_eq!(counts.get("if"), Some(&1));
    assert_eq!(counts.get("respond_to?"), Some(&1)); // Question mark method
    assert_eq!(counts.get("return"), Some(&1));
    assert_eq!(counts.get("true"), Some(&1));
    assert_eq!(counts.get("end"), Some(&2)); // Two end statements
  }

  #[test]
  fn test_exception_handling() {
    // Test exception handling keywords
    let content =
      "begin\n  raise StandardError\nrescue => e\n  retry\nensure\n  puts 'cleanup'\nend";
    let counts = count_keywords(content);
    assert_eq!(counts.get("begin"), Some(&1));
    assert_eq!(counts.get("raise"), Some(&1));
    assert_eq!(counts.get("rescue"), Some(&1));
    assert_eq!(counts.get("retry"), Some(&1));
    assert_eq!(counts.get("ensure"), Some(&1));
    assert_eq!(counts.get("end"), Some(&1));
  }
}
