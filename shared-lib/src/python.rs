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
    count_keywords(&content, total_counts);
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
        count_keywords(&content, total_counts);
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

fn count_keywords(content: &str, total_counts: &mut HashMap<String, usize>) {
  // Initialize all keywords with 0
  for keyword in PYTHON_KEYWORDS {
    total_counts.entry(keyword.to_string()).or_insert(0);
  }

  for keyword in PYTHON_KEYWORDS {
    let count = count_keyword(content, keyword);
    *total_counts.entry(keyword.to_string()).or_insert(0) += count;
  }
}

fn count_keyword(content: &str, keyword: &str) -> usize {
  if keyword.is_empty() || content.is_empty() {
    return 0;
  }

  let mut count = 0;
  let keyword_bytes = keyword.as_bytes();
  let content_bytes = content.as_bytes();

  if keyword_bytes.len() > content_bytes.len() {
    return 0;
  }

  let mut i = 0;
  while i <= content_bytes.len() - keyword_bytes.len() {
    // Check if we found the keyword
    if content_bytes[i..i + keyword_bytes.len()] == *keyword_bytes {
      // Check boundaries to ensure it's a whole word
      let is_start_boundary = i == 0 || !is_identifier_char(content_bytes[i - 1]);
      let is_end_boundary = i + keyword_bytes.len() >= content_bytes.len()
        || !is_identifier_char(content_bytes[i + keyword_bytes.len()]);

      if is_start_boundary && is_end_boundary {
        count += 1;
      }
    }
    i += 1;
  }

  count
}

fn is_identifier_char(c: u8) -> bool {
  c.is_ascii_alphanumeric() || c == b'_'
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
  fn test_count_keyword() {
    let content = "def hello():\n    print('Hello, world!')\n    return True";

    assert_eq!(count_keyword(content, "def"), 1);
    assert_eq!(count_keyword(content, "print"), 1);
    assert_eq!(count_keyword(content, "return"), 1);
    assert_eq!(count_keyword(content, "True"), 1);
    assert_eq!(count_keyword(content, "false"), 0); // Case sensitive
    assert_eq!(count_keyword(content, "class"), 0);
  }

  #[test]
  fn test_count_keyword_word_boundaries() {
    let content = "class MyClass:\n    def __init__(self):\n        self.class_name = 'test'";

    // Should find 'class' keyword but not in 'class_name'
    assert_eq!(count_keyword(content, "class"), 1);
    assert_eq!(count_keyword(content, "def"), 1);
    assert_eq!(count_keyword(content, "__init__"), 1);
    assert_eq!(count_keyword(content, "self"), 2);
  }

  #[test]
  fn test_count_keyword_edge_cases() {
    // Test empty content
    assert_eq!(count_keyword("", "def"), 0);

    // Test content without keywords
    assert_eq!(count_keyword("hello world", "def"), 0);

    // Test keyword at start/end
    assert_eq!(count_keyword("def", "def"), 1);
    assert_eq!(count_keyword("def func", "def"), 1);
    assert_eq!(count_keyword("func def", "def"), 1);

    // Test multiple occurrences
    assert_eq!(count_keyword("def f(): def g(): pass", "def"), 2);
    assert_eq!(count_keyword("def f(): def g(): pass", "pass"), 1);
  }

  #[test]
  fn test_is_identifier_char() {
    assert!(is_identifier_char(b'a'));
    assert!(is_identifier_char(b'Z'));
    assert!(is_identifier_char(b'0'));
    assert!(is_identifier_char(b'9'));
    assert!(is_identifier_char(b'_'));

    assert!(!is_identifier_char(b' '));
    assert!(!is_identifier_char(b'('));
    assert!(!is_identifier_char(b')'));
    assert!(!is_identifier_char(b':'));
    assert!(!is_identifier_char(b'.'));
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

    count_keywords(basic_content, &mut test_counts);

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

    count_keywords(built_in_content, &mut test_counts);

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
