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

  if path.is_file() && is_rust_file(&path) {
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
      } else if is_rust_file(&entry_path) {
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

pub fn is_rust_file(path: &Path) -> bool {
  path.extension().map_or(false, |ext| ext == "rs")
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
  let mut in_single_comment = false;
  let mut in_multi_comment = false;
  let mut string_char = '\0';

  while let Some(c) = chars.next() {
    match c {
      // Handle single-line comments
      '/' if !in_string && !in_multi_comment => {
        if chars.peek() == Some(&'/') {
          chars.next(); // consume second '/'
          in_single_comment = true;
          if !current_token.is_empty() {
            check_and_count_token(&current_token, &mut counts);
            current_token.clear();
          }
          continue;
        } else if chars.peek() == Some(&'*') {
          chars.next(); // consume '*'
          in_multi_comment = true;
          if !current_token.is_empty() {
            check_and_count_token(&current_token, &mut counts);
            current_token.clear();
          }
          continue;
        }
      }
      // End multi-line comment
      '*' if in_multi_comment => {
        if chars.peek() == Some(&'/') {
          chars.next(); // consume '/'
          in_multi_comment = false;
          continue;
        }
      }
      // End single-line comment
      '\n' if in_single_comment => {
        in_single_comment = false;
        continue;
      }
      // Handle strings (double quotes and single quotes for char literals)
      '"' | '\'' if !in_single_comment && !in_multi_comment => {
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
      _ if in_string || in_single_comment || in_multi_comment => {
        continue;
      }
      // Handle regular tokens
      _ => {
        if c.is_alphanumeric() || c == '_' {
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
  if RUST_KEYWORDS.contains(&token) {
    *counts.entry(token.to_string()).or_insert(0) += 1;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_count_keywords() {
    // Primitive Types and Literals
    let primitive_types = r#"
            fn demonstrate_primitives() {
                let int8: i8 = -128;
                let int16: i16 = -32768;
                let int32: i32 = -2147483648;
                let int64: i64 = -9223372036854775808;
                let int128: i128 = 0;
                let isize_val: isize = -1;
                
                let uint8: u8 = 255;
                let uint16: u16 = 65535;
                let uint32: u32 = 4294967295;
                let uint64: u64 = 18446744073709551615;
                let uint128: u128 = 0;
                let usize_val: usize = 1;
                
                let float32: f32 = 3.14;
                let float64: f64 = 2.718281828;
                let boolean: bool = true;
                let boolean_false: bool = false;
                let character: char = 'A';
                let string_slice: &str = "hello";
            }
        "#;

    // Control Flow
    let control_flow = r#"
            fn control_structures() {
                if condition {
                    // do something
                } else {
                    // do something else
                }
                
                match value {
                    1 => break,
                    2 => continue,
                    _ => {}
                }
                
                loop {
                    if should_exit {
                        break;
                    }
                    continue;
                }
                
                while condition {
                    // repeat
                }
                
                for item in collection {
                    // iterate
                }
            }
        "#;

    // Type Definitions and Structures
    let type_definitions = r#"
            struct Person {
                name: String,
                age: u32,
            }
            
            enum Color {
                Red,
                Green,
                Blue,
            }
            
            trait Drawable {
                fn draw(&self);
            }
            
            type UserId = u64;
        "#;

    // Functions and Control
    let functions = r#"
            fn basic_function() {
                return;
            }
            
            fn with_move_closure() {
                let closure = move |x| x + 1;
                return closure(5);
            }
        "#;

    // Visibility and Mutability
    let visibility_mutability = r#"
            pub struct PublicStruct {
                pub field: i32,
            }
            
            const CONSTANT: i32 = 42;
            static STATIC_VAR: i32 = 100;
            
            fn mutability_demo() {
                let immutable = 10;
                let mut mutable = 20;
                mutable += 1;
            }
        "#;

    // Module System and Scope
    let modules_scope = r#"
            mod my_module {
                use super::external_function;
                use crate::root_item;
                
                extern "C" {
                    fn c_function();
                }
                
                pub fn module_function() {
                    super::parent_function();
                    self::local_function();
                    Self::associated_function();
                }
            }
        "#;

    // Async and Concurrency
    let async_concurrency = r#"
            async fn async_function() {
                let result = other_async_function().await;
                result
            }
        "#;

    // Other Keywords and Operators
    let other_keywords = r#"
            fn other_features() {
                let x = value as i32;
                
                if let Some(val) = option {
                    // pattern matching
                }
                
                let numbers = vec![1, 2, 3];
                for num in numbers {
                    // iteration
                }
                
                let reference = &value;
                let ref_pattern = ref value;
                
                fn generic_function<T>() where T: Clone {
                    // generic with where clause
                }
                
                unsafe {
                    // unsafe code
                }
            }
        "#;

    // Reserved Keywords (future-proofing)
    let reserved_keywords = r#"
            fn reserved_demonstration() {
                // These are reserved for future use - testing them outside comments
                let _ = (abstract, become, box, do, final, macro, override, priv, try, typeof, unsized, virtual, yield);
            }
        "#;

    // Combine all sections
    let comprehensive_content = format!(
      "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
      primitive_types,
      control_flow,
      type_definitions,
      functions,
      visibility_mutability,
      modules_scope,
      async_concurrency,
      other_keywords,
      reserved_keywords
    );

    let counts = count_keywords(&comprehensive_content);

    // Verify that ALL keywords are present in the comprehensive example
    let mut missing_keywords = Vec::new();
    for &keyword in RUST_KEYWORDS {
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
    for &keyword in RUST_KEYWORDS {
      assert!(
        counts.get(keyword).unwrap_or(&0) >= &1,
        "Keyword '{}' should appear at least once in the comprehensive example",
        keyword
      );
    }

    println!(
      "✅ All {} Rust keywords are properly tested!",
      RUST_KEYWORDS.len()
    );

    // Test edge case: no keywords
    let no_keywords_content = "hello world 123 test";
    let no_keywords_counts = count_keywords(no_keywords_content);
    assert!(no_keywords_counts.is_empty());
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

    // Test keywords in comments (should NOT be counted with new implementation)
    let content = "// This is a fn comment with let and if";
    let counts = count_keywords(content);
    assert_eq!(counts.get("fn"), None); // Not counted in comment
    assert_eq!(counts.get("let"), None); // Not counted in comment
    assert_eq!(counts.get("if"), None); // Not counted in comment

    // Test keywords in strings (should NOT be counted with new implementation)
    let content = r#"let message = "This contains fn keyword";"#;
    let counts = count_keywords(content);
    assert_eq!(counts.get("let"), Some(&1)); // Only the actual let keyword
    assert_eq!(counts.get("fn"), None); // Not counted in string
  }

  #[test]
  fn test_string_and_comment_exclusion() {
    // Test single-line comments
    let content = "let x = 1; // This let should not be counted";
    let counts = count_keywords(content);
    assert_eq!(counts.get("let"), Some(&1)); // Only the actual let

    // Test multi-line comments
    let content = "let x = 1; /* let fn if */ fn test() {}";
    let counts = count_keywords(content);
    assert_eq!(counts.get("let"), Some(&1)); // Only the actual let
    assert_eq!(counts.get("fn"), Some(&1)); // Only the actual fn
    assert_eq!(counts.get("if"), None); // if is only in comment

    // Test various string types
    let content = r#"
      let single_quote = 'c';
      let double_quote = "This has let and fn in it";
      fn test() { return true; }
    "#;
    let counts = count_keywords(content);
    assert_eq!(counts.get("let"), Some(&2)); // Two let declarations
    assert_eq!(counts.get("fn"), Some(&1)); // One fn declaration
    assert_eq!(counts.get("return"), Some(&1)); // One return
    assert_eq!(counts.get("true"), Some(&1)); // One true

    // Test escape sequences in strings
    let content = r#"let message = "Quote: \"let x = 42;\""; fn main() {}"#;
    let counts = count_keywords(content);
    assert_eq!(counts.get("let"), Some(&1)); // Only the actual let declaration
    assert_eq!(counts.get("fn"), Some(&1)); // Only the actual fn declaration
  }

  #[test]
  fn test_partial_word_matches() {
    // Test that keywords within identifiers are NOT counted
    let content = "let lettering = 42; fn function_name() { return return_value; }";
    let counts = count_keywords(content);
    assert_eq!(counts.get("let"), Some(&1)); // Only the actual let keyword
    assert_eq!(counts.get("fn"), Some(&1)); // Only the actual fn keyword
    assert_eq!(counts.get("return"), Some(&1)); // Only the actual return keyword

    // These should NOT be counted as they are part of identifiers
    assert_eq!(counts.get("lettering"), None);
    assert_eq!(counts.get("function_name"), None);
    assert_eq!(counts.get("return_value"), None);

    // Test with type keywords
    let content = "struct MyStruct { bool_field: bool, string_type: str }";
    let counts = count_keywords(content);
    assert_eq!(counts.get("struct"), Some(&1));
    assert_eq!(counts.get("bool"), Some(&1));
    assert_eq!(counts.get("str"), Some(&1));
    // These should NOT be counted
    assert_eq!(counts.get("bool_field"), None);
    assert_eq!(counts.get("string_type"), None);
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

  #[test]
  fn test_is_rust_file() {
    use std::path::PathBuf;

    // Rust files should be detected
    assert!(is_rust_file(&PathBuf::from("main.rs")));
    assert!(is_rust_file(&PathBuf::from("lib.rs")));
    assert!(is_rust_file(&PathBuf::from("mod.rs")));
    assert!(is_rust_file(&PathBuf::from("src/main.rs")));
    assert!(is_rust_file(&PathBuf::from("/absolute/path/file.rs")));
    assert!(is_rust_file(&PathBuf::from("./relative/path/file.rs")));

    // Non-Rust files should not be detected
    assert!(!is_rust_file(&PathBuf::from("file.ts")));
    assert!(!is_rust_file(&PathBuf::from("file.js")));
    assert!(!is_rust_file(&PathBuf::from("file.py")));
    assert!(!is_rust_file(&PathBuf::from("file.txt")));
    assert!(!is_rust_file(&PathBuf::from("file.md")));
    assert!(!is_rust_file(&PathBuf::from("file")));
    assert!(!is_rust_file(&PathBuf::from("main.rs.bak")));
    assert!(!is_rust_file(&PathBuf::from("Cargo.toml")));
  }

  #[test]
  fn test_analyze_file_error_cases() {
    use std::path::PathBuf;
    let mut counts = HashMap::new();

    // Test with non-existent file
    let result = analyze_file(&PathBuf::from("non_existent_file.rs"), &mut counts);
    assert!(result.is_err());

    // Test with directory instead of file (should error)
    let result = analyze_file(&PathBuf::from("/"), &mut counts);
    assert!(result.is_err());
  }
}
