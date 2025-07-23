use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const JAVASCRIPT_KEYWORDS: &[&str] = &[
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

  if path.is_file() && is_javascript_file(&path) {
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
      } else if is_javascript_file(&entry_path) {
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

pub fn is_javascript_file(path: &Path) -> bool {
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
    if JAVASCRIPT_KEYWORDS.contains(&word) {
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
    // Import/Export and Module System
    let imports_exports = r#"
            import { Component } from 'react';
            export default class App extends Component {
                const fs = require('fs');
                
                declare module 'external-lib' {
                    export function method(): void;
                }
                
                declare package 'my-package' {
                    export const version: string;
                }
            }
        "#;

    // Classes and Object-Oriented Programming
    let classes_oop = r#"
            abstract class BaseEntity {
                private readonly id: number;
                protected name: string;
                public static count: number = 0;
                
                constructor(name: string) {
                    this.name = name;
                    this.id = BaseEntity.count++;
                }
                
                abstract process(): void;
                
                get getName(): string {
                    return this.name;
                }
                
                set setName(value: string) {
                    this.name = value;
                }
            }
            
            class UserService implements Service {
                override async fetchUser(id: bigint): Promise<User | null> {
                    const result = await super.fetchUser(id);
                    return result;
                }
            }
        "#;

    // Interface and Type Definitions
    let interfaces_types = r#"
            interface User {
                name: string;
                age: number;
                isActive: boolean;
                data: any;
                tags: symbol[];
                metadata: object;
                id: bigint;
            }
            
            type Status = 'active' | 'inactive';
            type UserKey = keyof User;
            type StringOrNumber = string | number;
            type ReadonlyUser = Readonly<User>;
            
            enum Color {
                Red,
                Green,
                Blue
            }
        "#;

    // Advanced TypeScript Types
    let advanced_types = r#"
            type NonNullable<T> = T extends null | undefined ? never : T;
            type ReturnType<T> = T extends (...args: any[]) => infer R ? R : never;
            type Contravariant<out T> = {
                consume: (value: T) => void;
            };
            type Uppercase<S extends string> = intrinsic;
            
            const config = {
                apiUrl: 'https://api.example.com',
                timeout: 5000
            } satisfies Config;
        "#;

    // Control Flow and Logic
    let control_flow = r#"
            function processUsers(): void {
                for (const user of users) {
                    if (user.isActive) {
                        switch (user.status) {
                            case 'active':
                                break;
                            case 'inactive':
                                break;
                            default:
                                debugger;
                                continue;
                        }
                    } else {
                        do {
                            user.retryCount++;
                        } while (user.retryCount < 3);
                    }
                }
            }
        "#;

    // Async/Await and Error Handling
    let async_error_handling = r#"
            async function fetchUser(id: bigint): Promise<User | null> {
                try {
                    const response = await fetch(`/api/user/${id}`);
                    if (!response.ok) {
                        throw new Error('Failed to fetch user');
                    }
                    return await response.json() as User;
                } catch (error) {
                    console.error('Error fetching user:', error);
                    return null;
                } finally {
                    console.log('Fetch attempt completed');
                }
            }
        "#;

    // Functions and Generators
    let functions_generators = r#"
            function isUser(obj: any): obj is User {
                return obj && typeof obj.name === 'string';
            }
            
            function assertUser(obj: unknown): asserts obj is User {
                if (!isUser(obj)) {
                    throw new Error('Not a user');
                }
            }
            
            function* generateUsers(): Generator<User, void, unknown> {
                for (let i = 0; i < users.length; i++) {
                    yield users[i];
                }
            }
            
            function logMessage(message: string): void {
                console.log(message);
            }
        "#;

    // Variables and Operators
    let variables_operators = r#"
            const API_URL: string = 'https://api.example.com';
            let currentUser: User | null = null;
            var globalConfig: any = {};
            
            const isReady: boolean = true;
            const isDisabled: boolean = false;
            const emptyValue: null = null;
            const notSet: undefined = undefined;
            const dynamicValue: unknown = 'could be anything';
            const neverValue: never = (() => { throw new Error(); })();
            const uniqueSymbol: unique symbol = Symbol('unique');
            
            if (user instanceof UserModel && typeof user.name === 'string') {
                return user.name in validNames && user.age > 0;
            }
            
            delete globalConfig.temporaryProperty;
        "#;

    // Namespace and Global Declarations
    let namespaces_globals = r#"
            namespace Utils {
                export function helper(): void {}
            }
            
            declare global {
                interface Window {
                    customProperty: unknown;
                }
            }
            
            var oldVar = 'legacy';
            with (config) {
                // Legacy code example
            }
        "#;

    // Combine all sections
    let comprehensive_content = format!(
      "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
      imports_exports,
      classes_oop,
      interfaces_types,
      advanced_types,
      control_flow,
      async_error_handling,
      functions_generators,
      variables_operators,
      namespaces_globals
    );

    let counts = count_keywords(&comprehensive_content);

    let mut missing_keywords = Vec::new();
    for &keyword in JAVASCRIPT_KEYWORDS {
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

    for &keyword in JAVASCRIPT_KEYWORDS {
      assert!(
        counts.get(keyword).unwrap_or(&0) >= &1,
        "Keyword '{}' should appear at least once in the comprehensive example",
        keyword
      );
    }

    println!(
      "✅ All {} JavaScript/TypeScript keywords are properly tested!",
      JAVASCRIPT_KEYWORDS.len()
    );

    let no_keywords_content = "hello world 123 test";
    let no_keywords_counts = count_keywords(no_keywords_content);
    assert!(no_keywords_counts.is_empty());
  }

  #[test]
  fn test_is_javascript_file() {
    use std::path::PathBuf;

    // JavaScript/TypeScript files should be detected
    assert!(is_javascript_file(&PathBuf::from("app.ts")));
    assert!(is_javascript_file(&PathBuf::from("component.tsx")));
    assert!(is_javascript_file(&PathBuf::from("script.js")));
    assert!(is_javascript_file(&PathBuf::from("component.jsx")));
    assert!(is_javascript_file(&PathBuf::from("path/to/file.ts")));

    // Non-JavaScript files should not be detected
    assert!(!is_javascript_file(&PathBuf::from("file.rs")));
    assert!(!is_javascript_file(&PathBuf::from("file.py")));
    assert!(!is_javascript_file(&PathBuf::from("file.txt")));
    assert!(!is_javascript_file(&PathBuf::from("file")));
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
  fn test_all_javascript_keywords_recognized() {
    // Test that all keywords in JAVASCRIPT_KEYWORDS are properly recognized
    for keyword in JAVASCRIPT_KEYWORDS {
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
