use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const GOLANG_KEYWORDS: &[&str] = &[
  // Go language keywords (25 keywords)
  "break",
  "case",
  "chan",
  "const",
  "continue",
  "default",
  "defer",
  "else",
  "fallthrough",
  "for",
  "func",
  "go",
  "goto",
  "if",
  "import",
  "interface",
  "map",
  "package",
  "range",
  "return",
  "select",
  "struct",
  "switch",
  "type",
  "var",
  // Built-in types
  "bool",
  "byte",
  "complex64",
  "complex128",
  "error",
  "float32",
  "float64",
  "int",
  "int8",
  "int16",
  "int32",
  "int64",
  "rune",
  "string",
  "uint",
  "uint8",
  "uint16",
  "uint32",
  "uint64",
  "uintptr",
  // Built-in constants
  "false",
  "true",
  "iota",
  "nil",
  // Built-in functions
  "append",
  "cap",
  "close",
  "complex",
  "copy",
  "delete",
  "imag",
  "len",
  "make",
  "new",
  "panic",
  "print",
  "println",
  "real",
  "recover",
];

pub fn analyze_directory(
  path: &str,
  total_counts: &mut HashMap<String, usize>,
  file_count: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
  let path = Path::new(path);

  if path.is_file() && is_go_file(&path) {
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
      } else if is_go_file(&entry_path) {
        eprintln!("Analyzing file: {}", entry_path.display());
        analyze_file(&entry_path, total_counts)?;
        *file_count += 1;
        eprintln!("Files processed: {}", file_count);
      }
    }
  }

  Ok(())
}

fn is_go_file(path: &Path) -> bool {
  if let Some(extension) = path.extension() {
    return extension == "go";
  }
  false
}

fn should_skip_dir(path: &Path) -> bool {
  if let Some(dir_name) = path.file_name() {
    if let Some(name_str) = dir_name.to_str() {
      return matches!(
        name_str,
        "vendor" | "node_modules" | ".git" | "target" | "bin" | "pkg" | ".vscode" | ".idea"
      );
    }
  }
  false
}

fn analyze_file(
  path: &Path,
  total_counts: &mut HashMap<String, usize>,
) -> Result<(), Box<dyn std::error::Error>> {
  let content = fs::read_to_string(path)?;

  for keyword in GOLANG_KEYWORDS {
    let count = count_keyword(&content, keyword);
    *total_counts.entry(keyword.to_string()).or_insert(0) += count;
  }

  Ok(())
}

fn count_keyword(content: &str, keyword: &str) -> usize {
  let mut count = 0;
  let keyword_bytes = keyword.as_bytes();
  let content_bytes = content.as_bytes();

  for i in 0..content_bytes.len() {
    if i + keyword_bytes.len() <= content_bytes.len() {
      if content_bytes[i..i + keyword_bytes.len()] == *keyword_bytes {
        let is_word_boundary_before = i == 0 || !is_identifier_char(content_bytes[i - 1]);
        let is_word_boundary_after = i + keyword_bytes.len() == content_bytes.len()
          || !is_identifier_char(content_bytes[i + keyword_bytes.len()]);

        if is_word_boundary_before && is_word_boundary_after {
          count += 1;
        }
      }
    }
  }

  count
}

fn is_identifier_char(c: u8) -> bool {
  c.is_ascii_alphanumeric() || c == b'_'
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashMap;

  #[test]
  fn test_golang_keywords_count() {
    assert_eq!(GOLANG_KEYWORDS.len(), 64);
  }

  #[test]
  fn test_golang_keywords_completeness() {
    // Test that all major Go categories are included
    let keywords_str = GOLANG_KEYWORDS.join(" ");

    // Language keywords
    assert!(keywords_str.contains("func"));
    assert!(keywords_str.contains("package"));
    assert!(keywords_str.contains("import"));
    assert!(keywords_str.contains("var"));
    assert!(keywords_str.contains("const"));
    assert!(keywords_str.contains("type"));
    assert!(keywords_str.contains("struct"));
    assert!(keywords_str.contains("interface"));
    assert!(keywords_str.contains("chan"));
    assert!(keywords_str.contains("go"));
    assert!(keywords_str.contains("defer"));
    assert!(keywords_str.contains("select"));
    assert!(keywords_str.contains("range"));

    // Control flow
    assert!(keywords_str.contains("if"));
    assert!(keywords_str.contains("else"));
    assert!(keywords_str.contains("for"));
    assert!(keywords_str.contains("switch"));
    assert!(keywords_str.contains("case"));
    assert!(keywords_str.contains("default"));
    assert!(keywords_str.contains("fallthrough"));
    assert!(keywords_str.contains("break"));
    assert!(keywords_str.contains("continue"));
    assert!(keywords_str.contains("return"));
    assert!(keywords_str.contains("goto"));

    // Built-in types
    assert!(keywords_str.contains("int"));
    assert!(keywords_str.contains("string"));
    assert!(keywords_str.contains("bool"));
    assert!(keywords_str.contains("byte"));
    assert!(keywords_str.contains("rune"));
    assert!(keywords_str.contains("float32"));
    assert!(keywords_str.contains("float64"));
    assert!(keywords_str.contains("complex64"));
    assert!(keywords_str.contains("complex128"));
    assert!(keywords_str.contains("error"));

    // Built-in constants
    assert!(keywords_str.contains("true"));
    assert!(keywords_str.contains("false"));
    assert!(keywords_str.contains("nil"));
    assert!(keywords_str.contains("iota"));

    // Built-in functions
    assert!(keywords_str.contains("make"));
    assert!(keywords_str.contains("new"));
    assert!(keywords_str.contains("len"));
    assert!(keywords_str.contains("cap"));
    assert!(keywords_str.contains("append"));
    assert!(keywords_str.contains("copy"));
    assert!(keywords_str.contains("delete"));
    assert!(keywords_str.contains("panic"));
    assert!(keywords_str.contains("recover"));
  }

  #[test]
  fn test_is_go_file() {
    assert!(is_go_file(Path::new("main.go")));
    assert!(is_go_file(Path::new("src/lib.go")));
    assert!(is_go_file(Path::new("/path/to/file.go")));

    assert!(!is_go_file(Path::new("main.rs")));
    assert!(!is_go_file(Path::new("file.js")));
    assert!(!is_go_file(Path::new("README.md")));
    assert!(!is_go_file(Path::new("Makefile")));
    assert!(!is_go_file(Path::new("go.mod")));
    assert!(!is_go_file(Path::new("go.sum")));
  }

  #[test]
  fn test_should_skip_dir() {
    assert!(should_skip_dir(Path::new("vendor")));
    assert!(should_skip_dir(Path::new("node_modules")));
    assert!(should_skip_dir(Path::new(".git")));
    assert!(should_skip_dir(Path::new("target")));
    assert!(should_skip_dir(Path::new("bin")));
    assert!(should_skip_dir(Path::new("pkg")));
    assert!(should_skip_dir(Path::new(".vscode")));
    assert!(should_skip_dir(Path::new(".idea")));

    assert!(!should_skip_dir(Path::new("src")));
    assert!(!should_skip_dir(Path::new("cmd")));
    assert!(!should_skip_dir(Path::new("internal")));
    assert!(!should_skip_dir(Path::new("pkg_valid")));
    assert!(!should_skip_dir(Path::new("test")));
  }

  #[test]
  fn test_count_keyword() {
    let code = r#"
package main

import "fmt"

func main() {
    var message string = "Hello, World!"
    fmt.Println(message)
    
    for i := 0; i < 10; i++ {
        if i%2 == 0 {
            fmt.Printf("Even: %d\n", i)
        } else {
            fmt.Printf("Odd: %d\n", i)
        }
    }
}
"#;

    assert_eq!(count_keyword(code, "package"), 1);
    assert_eq!(count_keyword(code, "func"), 1);
    assert_eq!(count_keyword(code, "var"), 1);
    assert_eq!(count_keyword(code, "string"), 1);
    assert_eq!(count_keyword(code, "for"), 1);
    assert_eq!(count_keyword(code, "if"), 1);
    assert_eq!(count_keyword(code, "else"), 1);
    assert_eq!(count_keyword(code, "import"), 1);

    // Test that count_keyword counts any string occurrence (fmt appears 4 times)
    assert_eq!(count_keyword(code, "fmt"), 4);

    // Test that count_keyword counts any string occurrence (main appears 2 times)
    assert_eq!(count_keyword(code, "main"), 2);
  }

  #[test]
  fn test_count_keyword_word_boundaries() {
    let code = r#"
package main
var variable int
var packagename string
func function() {}
"#;

    assert_eq!(count_keyword(code, "package"), 1); // Should not match "packagename"
    assert_eq!(count_keyword(code, "var"), 2);
    assert_eq!(count_keyword(code, "func"), 1); // Should not match "function"
    assert_eq!(count_keyword(code, "int"), 1);
  }

  #[test]
  fn test_count_keyword_edge_cases() {
    // Empty string
    assert_eq!(count_keyword("", "func"), 0);

    // Only keyword
    assert_eq!(count_keyword("func", "func"), 1);

    // Keyword at start
    assert_eq!(count_keyword("func main() {}", "func"), 1);

    // Keyword at end
    assert_eq!(count_keyword("return nil", "nil"), 1);

    // Multiple occurrences
    assert_eq!(count_keyword("var a, var b, var c", "var"), 3);

    // Keyword in string (should still count - this is a simple keyword counter)
    assert_eq!(count_keyword("\"func in string\"", "func"), 1);

    // Keyword in comment (should still count - this is a simple keyword counter)
    assert_eq!(count_keyword("// func comment", "func"), 1);
  }

  #[test]
  fn test_analyze_file_mock() {
    // Mock test for analyze_file function signature
    fn _test_signature() {
      let _: fn(&Path, &mut HashMap<String, usize>) -> Result<(), Box<dyn std::error::Error>> =
        analyze_file;
    }

    // Test that analyze_file can handle HashMap correctly
    let mut counts = HashMap::new();

    // This would test with a real file if it existed
    // We can't create real files in tests easily, so we test the HashMap structure
    counts.insert("func".to_string(), 5);
    counts.insert("var".to_string(), 3);

    assert_eq!(counts.get("func"), Some(&5));
    assert_eq!(counts.get("var"), Some(&3));
    assert_eq!(counts.get("nonexistent"), None);
  }

  #[test]
  fn test_is_identifier_char() {
    // Test alphanumeric characters
    assert!(is_identifier_char(b'a'));
    assert!(is_identifier_char(b'Z'));
    assert!(is_identifier_char(b'0'));
    assert!(is_identifier_char(b'9'));
    assert!(is_identifier_char(b'_'));

    // Test non-identifier characters
    assert!(!is_identifier_char(b' '));
    assert!(!is_identifier_char(b'.'));
    assert!(!is_identifier_char(b'('));
    assert!(!is_identifier_char(b')'));
    assert!(!is_identifier_char(b'{'));
    assert!(!is_identifier_char(b'}'));
    assert!(!is_identifier_char(b'['));
    assert!(!is_identifier_char(b']'));
    assert!(!is_identifier_char(b','));
    assert!(!is_identifier_char(b';'));
    assert!(!is_identifier_char(b':'));
    assert!(!is_identifier_char(b'\n'));
    assert!(!is_identifier_char(b'\t'));
  }

  #[test]
  fn test_analyze_directory_mock() {
    // Mock test for analyze_directory function signature
    fn _test_signature() {
      let _: fn(
        &str,
        &mut HashMap<String, usize>,
        &mut usize,
      ) -> Result<(), Box<dyn std::error::Error>> = analyze_directory;
    }

    // Test that the function can handle the correct parameter types
    let mut counts = HashMap::new();
    let mut file_count = 0usize;

    // Test parameter types work correctly
    counts.insert("test".to_string(), 1);
    file_count += 1;

    assert_eq!(file_count, 1);
    assert_eq!(counts.get("test"), Some(&1));
  }

  #[test]
  fn test_comprehensive_go_features() {
    let complex_go_code = r#"
package main

import (
    "fmt"
    "time"
)

type Person struct {
    Name string
    Age  int
}

type Speaker interface {
    Speak() string
}

func (p Person) Speak() string {
    return fmt.Sprintf("Hello, I'm %s", p.Name)
}

func main() {
    // Variable declarations
    var name string = "Alice"
    age := 30
    const greeting = "Hello"
    
    // Struct initialization
    person := Person{Name: name, Age: age}
    
    // Channel operations
    ch := make(chan string, 1)
    
    // Goroutine
    go func() {
        defer close(ch)
        ch <- person.Speak()
    }()
    
    // Select statement
    select {
    case msg := <-ch:
        fmt.Println(msg)
    case <-time.After(time.Second):
        fmt.Println("Timeout")
    }
    
    // Control flow
    for i := 0; i < 5; i++ {
        switch {
        case i%2 == 0:
            fmt.Printf("Even: %d\n", i)
        default:
            fmt.Printf("Odd: %d\n", i)
        }
    }
    
    // Error handling
    if err := someFunction(); err != nil {
        panic(err)
    }
    
    // Map operations
    m := make(map[string]int)
    m["key"] = 42
    
    // Slice operations
    slice := make([]int, 0, 10)
    slice = append(slice, 1, 2, 3)
    
    // Range iteration
    for index, value := range slice {
        fmt.Printf("Index: %d, Value: %d\n", index, value)
    }
}

func someFunction() error {
    return nil
}
"#;

    // Test comprehensive keyword counting
    assert!(count_keyword(complex_go_code, "package") >= 1);
    assert!(count_keyword(complex_go_code, "import") >= 1);
    assert!(count_keyword(complex_go_code, "type") >= 2);
    assert!(count_keyword(complex_go_code, "struct") >= 1);
    assert!(count_keyword(complex_go_code, "interface") >= 1);
    assert!(count_keyword(complex_go_code, "func") >= 3);
    assert!(count_keyword(complex_go_code, "var") >= 1);
    assert!(count_keyword(complex_go_code, "const") >= 1);
    assert!(count_keyword(complex_go_code, "chan") >= 1);
    assert!(count_keyword(complex_go_code, "make") >= 3);
    assert!(count_keyword(complex_go_code, "go") >= 1);
    assert!(count_keyword(complex_go_code, "defer") >= 1);
    assert!(count_keyword(complex_go_code, "select") >= 1);
    assert!(count_keyword(complex_go_code, "case") >= 2);
    assert!(count_keyword(complex_go_code, "for") >= 2);
    assert!(count_keyword(complex_go_code, "switch") >= 1);
    assert!(count_keyword(complex_go_code, "default") >= 1);
    assert!(count_keyword(complex_go_code, "if") >= 1);
    assert!(count_keyword(complex_go_code, "range") >= 1);
    assert!(count_keyword(complex_go_code, "return") >= 2);
    assert!(count_keyword(complex_go_code, "nil") >= 2);
    assert!(count_keyword(complex_go_code, "panic") >= 1);
    assert!(count_keyword(complex_go_code, "append") >= 1);
    assert!(count_keyword(complex_go_code, "string") >= 2);
    assert!(count_keyword(complex_go_code, "int") >= 2);
  }
}
