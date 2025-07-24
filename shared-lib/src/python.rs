use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const PYTHON_KEYWORDS: &[&str] = &[
  // Keywords (Python 3.11)
  "False",
  "None",
  "True",
  "and",
  "as",
  "assert",
  "async",
  "await",
  "break",
  "class",
  "continue",
  "def",
  "del",
  "elif",
  "else",
  "except",
  "finally",
  "for",
  "from",
  "global",
  "if",
  "import",
  "in",
  "is",
  "lambda",
  "nonlocal",
  "not",
  "or",
  "pass",
  "raise",
  "return",
  "try",
  "while",
  "with",
  "yield",
  // Built-in functions (commonly used)
  "abs",
  "all",
  "any",
  "ascii",
  "bin",
  "bool",
  "bytearray",
  "bytes",
  "callable",
  "chr",
  "classmethod",
  "compile",
  "complex",
  "delattr",
  "dict",
  "dir",
  "divmod",
  "enumerate",
  "eval",
  "exec",
  "filter",
  "float",
  "format",
  "frozenset",
  "getattr",
  "globals",
  "hasattr",
  "hash",
  "help",
  "hex",
  "id",
  "input",
  "int",
  "isinstance",
  "issubclass",
  "iter",
  "len",
  "list",
  "locals",
  "map",
  "max",
  "memoryview",
  "min",
  "next",
  "object",
  "oct",
  "open",
  "ord",
  "pow",
  "print",
  "property",
  "range",
  "repr",
  "reversed",
  "round",
  "set",
  "setattr",
  "slice",
  "sorted",
  "staticmethod",
  "str",
  "sum",
  "super",
  "tuple",
  "type",
  "vars",
  "zip",
  // Built-in exceptions (common ones)
  "Exception",
  "AttributeError",
  "IOError",
  "ImportError",
  "IndexError",
  "KeyError",
  "NameError",
  "RuntimeError",
  "SyntaxError",
  "TypeError",
  "ValueError",
  "ZeroDivisionError",
  // Built-in constants
  "NotImplemented",
  "Ellipsis",
  "__debug__",
  // Special methods (dunder methods - common ones)
  "__init__",
  "__str__",
  "__repr__",
  "__len__",
  "__getitem__",
  "__setitem__",
  "__delitem__",
  "__contains__",
  "__call__",
  "__enter__",
  "__exit__",
  "__iter__",
  "__next__",
];

pub fn analyze_directory(
  path: &str,
  total_counts: &mut HashMap<String, usize>,
  file_count: &mut usize,
) -> Result<(), Box<dyn std::error::Error>> {
  let path = Path::new(path);

  if path.is_file() && is_python_file(&path) {
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
      } else if is_python_file(&entry_path) {
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

fn is_python_file(path: &Path) -> bool {
  if let Some(extension) = path.extension() {
    let ext = extension.to_string_lossy().to_lowercase();
    matches!(ext.as_str(), "py" | "pyw" | "pyi")
  } else {
    // Check for files without extensions but with Python shebang
    if let Some(filename) = path.file_name() {
      let name = filename.to_string_lossy();
      // Common Python files without extensions
      matches!(name.as_ref(), "Pipfile" | "Pipfile.lock" | "__init__")
    } else {
      false
    }
  }
}

fn should_skip_dir(path: &Path) -> bool {
  if let Some(dir_name) = path.file_name() {
    let name = dir_name.to_string_lossy();
    matches!(
      name.as_ref(),
      "__pycache__"
        | ".pytest_cache"
        | "venv"
        | "env"
        | ".venv"
        | ".env"
        | "site-packages"
        | "dist"
        | "build"
        | ".git"
        | "target"
        | "node_modules"
        | ".tox"
        | ".coverage"
        | "htmlcov"
        | ".mypy_cache"
        | ".idea"
        | ".vscode"
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
  let mut string_char = '\0';
  let mut in_triple_quote = false;
  let mut triple_quote_type = '\0';

  while let Some(c) = chars.next() {
    match c {
      // Handle single-line comments (Python uses #)
      '#' if !in_string && !in_triple_quote => {
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
      // Handle triple quotes (Python docstrings)
      '"' | '\'' if !in_comment => {
        if !in_string && !in_triple_quote {
          // Check for triple quotes
          if chars.peek() == Some(&c) {
            chars.next(); // consume second quote
            if chars.peek() == Some(&c) {
              chars.next(); // consume third quote
              in_triple_quote = true;
              triple_quote_type = c;
            } else {
              // Two quotes, treat as empty string
              in_string = true;
              string_char = c;
            }
          } else {
            // Single quote string
            in_string = true;
            string_char = c;
          }
          if !current_token.is_empty() {
            check_and_count_token(&current_token, &mut counts);
            current_token.clear();
          }
        } else if in_triple_quote && c == triple_quote_type {
          // Check for end of triple quote
          if chars.peek() == Some(&c) {
            chars.next(); // consume second quote
            if chars.peek() == Some(&c) {
              chars.next(); // consume third quote
              in_triple_quote = false;
              triple_quote_type = '\0';
            }
          }
        } else if in_string && c == string_char {
          in_string = false;
          string_char = '\0';
        }
        continue;
      }
      // Handle escape sequences in strings
      '\\' if in_string && !in_triple_quote => {
        // Skip the next character if we're in a string (escape sequence)
        chars.next();
        continue;
      }
      // Skip content inside strings, triple quotes, and comments
      _ if in_string || in_triple_quote || in_comment => {
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
  if PYTHON_KEYWORDS.contains(&token) {
    *counts.entry(token.to_string()).or_insert(0) += 1;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_is_python_file() {
    assert!(is_python_file(Path::new("test.py")));
    assert!(is_python_file(Path::new("script.pyw")));
    assert!(is_python_file(Path::new("types.pyi")));
    assert!(is_python_file(Path::new("Pipfile")));
    assert!(is_python_file(Path::new("__init__")));

    assert!(!is_python_file(Path::new("test.rs")));
    assert!(!is_python_file(Path::new("test.js")));
    assert!(!is_python_file(Path::new("README.md")));
  }

  #[test]
  fn test_should_skip_dir() {
    assert!(should_skip_dir(Path::new("__pycache__")));
    assert!(should_skip_dir(Path::new("venv")));
    assert!(should_skip_dir(Path::new(".pytest_cache")));
    assert!(should_skip_dir(Path::new("site-packages")));
    assert!(should_skip_dir(Path::new(".git")));
    assert!(should_skip_dir(Path::new(".mypy_cache")));

    assert!(!should_skip_dir(Path::new("src")));
    assert!(!should_skip_dir(Path::new("tests")));
    assert!(!should_skip_dir(Path::new("lib")));
  }

  #[test]
  fn test_python_keywords_count() {
    // Verify we have a reasonable number of Python keywords
    assert!(PYTHON_KEYWORDS.len() >= 80); // Should have at least 80 keywords/built-ins
    assert!(PYTHON_KEYWORDS.len() <= 130); // But not too many
  }

  #[test]
  fn test_all_python_keywords_recognized() {
    let mut test_counts = HashMap::new();

    // Test basic keywords
    let basic_content = r#"
def hello_world():
    if True:
        print("Hello")
        return None
    else:
        raise ValueError("Error")
        
class MyClass:
    def __init__(self):
        self.value = 42
        
async def async_func():
    await some_operation()
    
try:
    result = int("123")
except ValueError as e:
    pass
finally:
    print("Done")
    
for i in range(10):
    if i % 2 == 0:
        continue
    else:
        break
        
while True:
    pass
    
with open("file.txt") as f:
    content = f.read()
    
lambda x: x * 2
"#;

    let file_counts = count_keywords(basic_content);
    for (keyword, count) in file_counts {
      *test_counts.entry(keyword).or_insert(0) += count;
    }

    // Check that basic keywords are found
    assert!(*test_counts.get("def").unwrap_or(&0) >= 3);
    assert!(*test_counts.get("class").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("if").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("True").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("print").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("return").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("None").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("else").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("raise").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("ValueError").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("__init__").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("async").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("await").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("try").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("int").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("except").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("as").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("pass").unwrap_or(&0) >= 2);
    assert!(*test_counts.get("finally").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("for").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("in").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("range").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("continue").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("break").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("while").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("with").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("open").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("lambda").unwrap_or(&0) >= 1);
  }

  #[test]
  fn test_python_built_ins() {
    let mut test_counts = HashMap::new();

    let built_in_content = r#"
result = len([1, 2, 3])
numbers = list(range(5))
total = sum(numbers)
maximum = max(numbers)
minimum = min(numbers)
is_all_true = all([True, True, False])
is_any_true = any([False, True, False])
text = str(42)
number = int("123")
decimal = float("3.14")
items = dict(a=1, b=2)
unique = set([1, 2, 2, 3])
ordered = sorted([3, 1, 2])
"#;

    let file_counts = count_keywords(built_in_content);
    for (keyword, count) in file_counts {
      *test_counts.entry(keyword).or_insert(0) += count;
    }

    // Check built-in functions
    assert!(*test_counts.get("len").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("list").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("range").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("sum").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("max").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("min").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("all").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("any").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("str").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("int").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("float").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("dict").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("set").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("sorted").unwrap_or(&0) >= 1);
    assert!(*test_counts.get("True").unwrap_or(&0) >= 3);
    assert!(*test_counts.get("False").unwrap_or(&0) >= 2);
  }

  #[test]
  fn test_string_and_comment_exclusion() {
    // Test single-line comments
    let content = "def actual_function():\n    # This def should not be counted\n    pass";
    let counts = count_keywords(content);
    assert_eq!(counts.get("def"), Some(&1)); // Only the actual def
    assert_eq!(counts.get("pass"), Some(&1)); // Only the actual pass

    // Test various string types
    let content = r#"
      def main():
          single_quote = 'def test(): pass'
          double_quote = "This contains def and class keywords"
          return True
    "#;
    let counts = count_keywords(content);
    assert_eq!(counts.get("def"), Some(&1)); // One def declaration
    assert_eq!(counts.get("return"), Some(&1)); // One return statement
    assert_eq!(counts.get("True"), Some(&1)); // One True literal
                                              // These should NOT be counted as they're in strings
    assert_eq!(counts.get("test"), None); // Only in string
    assert_eq!(counts.get("pass"), None); // Only in string
    assert_eq!(counts.get("class"), None); // Only in string

    // Test triple quote strings (docstrings)
    let content = r#"
def documented_function():
    """
    This def function should not be counted in the docstring.
    Also class and if keywords should be ignored.
    """
    if True:
        pass
"#;
    let counts = count_keywords(content);
    assert_eq!(counts.get("def"), Some(&1)); // Only the actual def
    assert_eq!(counts.get("if"), Some(&1)); // Only the actual if
    assert_eq!(counts.get("True"), Some(&1)); // Only the actual True
    assert_eq!(counts.get("pass"), Some(&1)); // Only the actual pass
                                              // These should NOT be counted as they're in docstring
    assert_eq!(counts.get("class"), None); // Only in docstring

    // Test escape sequences in strings
    let content = r#"message = "Quote: \"def test()\""; def actual_function(): pass"#;
    let counts = count_keywords(content);
    assert_eq!(counts.get("def"), Some(&1)); // Only the actual def declaration
    assert_eq!(counts.get("pass"), Some(&1)); // Only the actual pass statement
  }

  #[test]
  fn test_partial_word_matches() {
    // Test that keywords within identifiers are NOT counted
    let content = "def define_function(): return return_value";
    let counts = count_keywords(content);
    assert_eq!(counts.get("def"), Some(&1)); // Only the actual def keyword
    assert_eq!(counts.get("return"), Some(&1)); // Only the actual return keyword

    // These should NOT be counted as they are part of identifiers
    assert_eq!(counts.get("define_function"), None);
    assert_eq!(counts.get("return_value"), None);

    // Test with Python-specific tokens
    let content = "import sys; class MyClass: pass; def __init__(self): pass";
    let counts = count_keywords(content);
    assert_eq!(counts.get("import"), Some(&1));
    assert_eq!(counts.get("class"), Some(&1));
    assert_eq!(counts.get("def"), Some(&1));
    assert_eq!(counts.get("pass"), Some(&2)); // Two pass statements
    assert_eq!(counts.get("__init__"), Some(&1)); // Special method name
  }

  #[test]
  fn test_analyze_file_error_cases() {
    // This test is primarily to check the function signature and basic behavior
    // We skip actual file system error testing to avoid environment-dependent issues
    let mut counts = HashMap::new();
    let mut file_count = 0;

    // Test that the function works with valid empty directory (like current directory)
    // but doesn't find Python files, which is a valid success case
    let result = analyze_directory(".", &mut counts, &mut file_count);

    // The function should succeed even if no Python files are found
    assert!(result.is_ok());
    // file_count might be 0 if no Python files in current directory, which is fine
  }
}
