# TODO - Rust Primitive Keyword Analysis

## Project Goal
Analyze `.rs` files to count usage frequency of primitive keywords.

## Tasks
- [ ] Define list of Rust primitive keywords to analyze
- [ ] Implement file scanner to find all `.rs` files
- [ ] Create keyword counter/parser
- [ ] Generate usage statistics report
- [ ] Add command line interface
- [ ] Add output formatting options (JSON, CSV, plain text)

## Keywords to Track

### Primitive Types
- [ ] `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
- [ ] `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- [ ] `f32`, `f64`
- [ ] `bool`
- [ ] `char`
- [ ] `str`

### Control Flow Keywords
- [ ] `if`, `else`, `match`, `break`, `continue`, `loop`, `while`, `for`

### Type Definition Keywords
- [ ] `struct`, `enum`, `trait`, `type`

### Function-related Keywords
- [ ] `fn`, `return`, `move`

### Visibility/Mutability Keywords
- [ ] `pub`, `mut`, `const`, `static`

### Module/Scope Keywords
- [ ] `mod`, `use`, `crate`, `extern`, `super`, `self`, `Self`

### Concurrency Keywords
- [ ] `async`, `await`

### Other Keywords
- [ ] `as`, `in`, `let`, `ref`, `where`, `unsafe`, `true`, `false`

### Reserved Keywords (Future Use)
- [ ] `abstract`, `become`, `box`, `do`, `final`, `macro`, `override`, `priv`, `try`, `typeof`, `unsized`, `virtual`, `yield`

## Optional Features
- [ ] Support for custom keyword lists
- [ ] Directory recursion options
- [ ] File filtering by patterns
- [ ] Export results to different formats