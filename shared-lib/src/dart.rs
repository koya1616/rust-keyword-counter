use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const DART_KEYWORDS: &[&str] = &[
  // Reserved words (cannot be used as identifiers)
  "abstract",
  "as",
  "assert",
  "async",
  "await",
  "base",
  "break",
  "case",
  "catch",
  "class",
  "const",
  "continue",
  "covariant",
  "default",
  "deferred",
  "do",
  "dynamic",
  "else",
  "enum",
  "export",
  "extends",
  "extension",
  "external",
  "factory",
  "false",
  "final",
  "finally",
  "for",
  "Function",
  "get",
  "hide",
  "if",
  "implements",
  "import",
  "in",
  "interface",
  "is",
  "late",
  "library",
  "mixin",
  "new",
  "null",
  "of",
  "on",
  "operator",
  "part",
  "required",
  "rethrow",
  "return",
  "sealed",
  "set",
  "show",
  "static",
  "super",
  "switch",
  "sync",
  "this",
  "throw",
  "true",
  "try",
  "type",
  "typedef",
  "var",
  "void",
  "when",
  "with",
  "while",
  "yield",
  // Built-in types and commonly used classes
  "int",
  "double",
  "num",
  "bool",
  "String",
  "List",
  "Map",
  "Set",
  "Object",
  "Null",
];

pub fn analyze_directory(
  path: &str,
  total_counts: &mut HashMap<String, usize>,
  file_count: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
  let path = Path::new(path);

  if path.is_file() && is_dart_file(path) {
    eprintln!("Analyzing file: {}", path.display());
    let content = fs::read_to_string(path)?;
    let file_counts = count_keywords(&content);

    for (keyword, count) in file_counts {
      *total_counts.entry(keyword).or_insert(0) += count;
    }
    *file_count += 1;
    eprintln!("Files processed: {}", *file_count);
  } else if path.is_dir() {
    let entries = fs::read_dir(path)?;
    for entry in entries {
      let entry = entry?;
      let entry_path = entry.path();

      if entry_path.is_dir() {
        if should_skip_dir(&entry_path) {
          continue;
        }
        eprintln!("Entering directory: {}", entry_path.display());
        analyze_directory(entry_path.to_str().unwrap(), total_counts, file_count)?;
      } else if is_dart_file(&entry_path) {
        eprintln!("Analyzing file: {}", entry_path.display());
        let content = fs::read_to_string(&entry_path)?;
        let file_counts = count_keywords(&content);

        for (keyword, count) in file_counts {
          *total_counts.entry(keyword).or_insert(0) += count;
        }
        *file_count += 1;
        eprintln!("Files processed: {}", *file_count);
      }
    }
  }

  Ok(())
}

fn is_dart_file(path: &Path) -> bool {
  if let Some(extension) = path.extension() {
    let ext = extension.to_string_lossy().to_lowercase();
    ext == "dart"
  } else {
    false
  }
}

fn should_skip_dir(path: &Path) -> bool {
  if let Some(dir_name) = path.file_name() {
    let name = dir_name.to_string_lossy();
    matches!(
      name.as_ref(),
      ".dart_tool" | "build" | ".pub" | ".git" | "target" | "node_modules" | ".idea" | ".vscode"
    )
  } else {
    false
  }
}

pub fn count_keywords(content: &str) -> HashMap<String, usize> {
  let mut counts = HashMap::new();
  let mut chars = content.chars().peekable();
  let mut current_token = String::new();
  let mut in_string = false;
  let mut in_comment = false;
  let mut in_multi_line_comment = false;
  let mut string_char = '\0';

  while let Some(c) = chars.next() {
    match c {
      // Handle multi-line comments (/* ... */)
      '/' if !in_string && !in_comment && !in_multi_line_comment => {
        if chars.peek() == Some(&'*') {
          chars.next(); // consume '*'
          in_multi_line_comment = true;
          if !current_token.is_empty() {
            check_and_count_token(&current_token, &mut counts);
            current_token.clear();
          }
          continue;
        } else if chars.peek() == Some(&'/') {
          chars.next(); // consume second '/'
          in_comment = true;
          if !current_token.is_empty() {
            check_and_count_token(&current_token, &mut counts);
            current_token.clear();
          }
          continue;
        }
      }
      // End multi-line comment
      '*' if in_multi_line_comment && !in_string => {
        if chars.peek() == Some(&'/') {
          chars.next(); // consume '/'
          in_multi_line_comment = false;
        }
        continue;
      }
      // End single-line comment
      '\n' if in_comment => {
        in_comment = false;
        continue;
      }
      // Handle strings
      '"' | '\'' if !in_comment && !in_multi_line_comment => {
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
      _ if in_string || in_comment || in_multi_line_comment => {
        continue;
      }
      // Handle regular tokens
      _ => {
        if c.is_alphanumeric() || c == '_' {
          current_token.push(c);
        } else if !current_token.is_empty() {
          check_and_count_token(&current_token, &mut counts);
          current_token.clear();
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
  if DART_KEYWORDS.contains(&token) {
    *counts.entry(token.to_string()).or_insert(0) += 1;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_dart_file() {
    assert!(is_dart_file(Path::new("main.dart")));
    assert!(is_dart_file(Path::new("app.dart")));
    assert!(is_dart_file(Path::new("lib/widget.dart")));

    assert!(!is_dart_file(Path::new("test.rs")));
    assert!(!is_dart_file(Path::new("test.js")));
    assert!(!is_dart_file(Path::new("README.md")));
  }

  #[test]
  fn test_should_skip_dir() {
    assert!(should_skip_dir(Path::new(".dart_tool")));
    assert!(should_skip_dir(Path::new("build")));
    assert!(should_skip_dir(Path::new(".pub")));
    assert!(should_skip_dir(Path::new(".git")));
    assert!(should_skip_dir(Path::new("node_modules")));

    assert!(!should_skip_dir(Path::new("lib")));
    assert!(!should_skip_dir(Path::new("test")));
    assert!(!should_skip_dir(Path::new("bin")));
  }

  #[test]
  fn test_dart_keywords_count() {
    // Verify we have the correct number of Dart keywords
    assert!(DART_KEYWORDS.len() >= 73); // Should have at least 73 keywords
    assert!(DART_KEYWORDS.len() <= 85); // But not too many
  }

  #[test]
  fn test_all_dart_keywords_recognized() {
    let mut test_counts = HashMap::new();

    let basic_content = r#"
void main() {
  if (true) {
    print('Hello');
    return;
  } else {
    throw Exception('Error');
  }
}

class MyClass {
  int value = 0;

  MyClass() {
    this.value = 42;
  }

  void increment() {
    value++;
  }
}

abstract class Animal {
  void makeSound();
}

class Dog extends Animal {
  @override
  void makeSound() {
    print('Woof');
  }
}

Future<void> asyncFunc() async {
  await Future.delayed(Duration(seconds: 1));
  print('Done');
}

void controlFlow() {
  for (var i = 0; i < 10; i++) {
    if (i % 2 == 0) {
      continue;
    } else {
      break;
    }
  }

  while (true) {
    break;
  }

  switch (value) {
    case 1:
      print('One');
      break;
    default:
      print('Other');
  }

  try {
    var result = int.parse('123');
  } catch (e) {
    print(e);
  } finally {
    print('Cleanup');
  }
}

enum Color { red, green, blue }

typedef IntCallback = void Function(int);
"#;

    let file_counts = count_keywords(basic_content);
    for (keyword, count) in file_counts {
      *test_counts.entry(keyword).or_insert(0) += count;
    }

    // Check that basic keywords are found
    assert!(*test_counts.get("void").unwrap_or(&0) >= 4);
    assert!(*test_counts.get("class").unwrap_or(&0) >= 3);
    assert!(*test_counts.get("if").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("true").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("return").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("else").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("throw").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("int").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("this").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("abstract").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("extends").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("async").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("await").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("for").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("var").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("continue").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("break").unwrap_or(&0) >= 3);
    assert!(*test_counts.get("while").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("switch").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("case").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("default").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("try").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("catch").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("finally").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("enum").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("typedef").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("Function").unwrap_or(&0) >= 1);
  }

  #[test]
  fn test_dart_built_ins() {
    let mut test_counts = HashMap::new();

    let built_in_content = r#"
void main() {
  int number = 42;
  double decimal = 3.14;
  num value = 10;
  bool isTrue = true;
  String text = 'Hello';
  List<int> numbers = [1, 2, 3];
  Map<String, int> scores = {'Alice': 100};
  Set<String> unique = {'a', 'b', 'c'};
  Object obj = Object();
  Null nothing = null;
}
"#;

    let file_counts = count_keywords(built_in_content);
    for (keyword, count) in file_counts {
      *test_counts.entry(keyword).or_insert(0) += count;
    }

    // Check built-in types
    assert!(*test_counts.get("void").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("int").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("double").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("num").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("bool").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("String").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("List").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("Map").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("Set").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("Object").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("Null").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("true").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("null").unwrap_or(&0) >= 1);
  }

  #[test]
  fn test_string_and_comment_exclusion() {
    // Test single-line comments
    let content = "void actualFunction() {\n  // This void should not be counted\n  return;\n}";
    let counts = count_keywords(content);
    assert_eq!(counts.get("void"), Some(&1)); // Only the actual void
    assert_eq!(counts.get("return"), Some(&1)); // Only the actual return

    // Test various string types
    let content = r#"
      void main() {
        String singleQuote = 'void test() { return; }';
        String doubleQuote = "This contains class and if keywords";
        return;
      }
    "#;
    let counts = count_keywords(content);
    assert_eq!(counts.get("void"), Some(&1)); // One void declaration
    assert_eq!(counts.get("return"), Some(&1)); // One return statement
    assert_eq!(counts.get("String"), Some(&2)); // Two String declarations
                                                // These should NOT be counted as they're in strings
    assert_eq!(counts.get("test"), None); // Only in string
    assert_eq!(counts.get("class"), None); // Only in string

    // Test multi-line comments
    let content = r#"
void documentedFunction() {
  /*
   * This void function should not be counted in the comment.
   * Also class and if keywords should be ignored.
   */
  if (true) {
    return;
  }
}
"#;
    let counts = count_keywords(content);
    assert_eq!(counts.get("void"), Some(&1)); // Only the actual void
    assert_eq!(counts.get("if"), Some(&1)); // Only the actual if
    assert_eq!(counts.get("true"), Some(&1)); // Only the actual true
    assert_eq!(counts.get("return"), Some(&1)); // Only the actual return
                                                // These should NOT be counted as they're in comment
    assert_eq!(counts.get("class"), None); // Only in comment

    // Test escape sequences in strings
    let content = r#"String message = "Quote: \"void test()\""; void actualFunction() { return; }"#;
    let counts = count_keywords(content);
    assert_eq!(counts.get("String"), Some(&1)); // String type
    assert_eq!(counts.get("void"), Some(&1)); // Only the actual void declaration
    assert_eq!(counts.get("return"), Some(&1)); // Only the actual return statement
  }

  #[test]
  fn test_partial_word_matches() {
    // Test that keywords within identifiers are NOT counted
    let content = "void defineFunction() { return returnValue; }";
    let counts = count_keywords(content);
    assert_eq!(counts.get("void"), Some(&1)); // Only the actual void keyword
    assert_eq!(counts.get("return"), Some(&1)); // Only the actual return keyword

    // These should NOT be counted as they are part of identifiers
    assert_eq!(counts.get("defineFunction"), None);
    assert_eq!(counts.get("returnValue"), None);

    // Test with Dart-specific tokens
    let content = "import 'package:flutter/material.dart'; class MyWidget extends StatelessWidget { @override Widget build(BuildContext context) { return Container(); } }";
    let counts = count_keywords(content);
    assert_eq!(counts.get("import"), Some(&1));
    assert_eq!(counts.get("class"), Some(&1));
    assert_eq!(counts.get("extends"), Some(&1));
    assert_eq!(counts.get("return"), Some(&1));
  }

  #[test]
  fn test_analyze_file_error_cases() {
    let mut counts = HashMap::new();
    let mut file_count = 0;

    // Test that the function works with valid directory
    let result = analyze_directory(".", &mut counts, &mut file_count);

    // The function should succeed even if no Dart files are found
    assert!(result.is_ok());
  }
}
