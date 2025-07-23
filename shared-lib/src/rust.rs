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
            // These are reserved for future use and should be recognized
            // abstract become box do final macro override priv try typeof unsized virtual yield
            // Note: These appear in comments since they're not yet implemented
            fn reserved_demonstration() {
                // abstract - for abstract classes/methods
                // become - for moves that destructure
                // box - for box syntax  
                // do - for do expressions
                // final - for final classes/methods
                // macro - for macro definitions
                // override - for method overriding
                // priv - for private visibility
                // try - for try expressions
                // typeof - for typeof operator
                // unsized - for unsized types
                // virtual - for virtual methods
                // yield - for generator functions
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
