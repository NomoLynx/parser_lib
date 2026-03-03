# parser_lib

`parser_lib` is a Rust crate in the `CLR` workspace that provides parser and conversion utilities used by the RISC-V toolchain.

It contains Pest-based parsers for multiple text formats (INI, CSV, JSON, Markdown, Mermaid variants), plus string-format processing helpers and shared parsing error models.

## Location

- Crate path: `CLR/parser_lib`
- Workspace root: `CLR/Cargo.toml`

## What this crate provides

### Core parsing modules

- `ini`
  - Parse `.ini` content and files
  - Flatten section + property data
  - Generate ASM `.data` string declarations
- `csv`
  - Parse CSV text or files into typed structures
  - Validate column uniqueness
  - Infer column types used by conversion helpers
- `json`
  - Parse JSON text or files
  - Infer Rust-oriented type shapes from JSON values
- `markdown_lang`
  - Parse Markdown documents into a structured model
  - Query headers, code blocks, tables, and footnotes

### Mermaid parsers

- `mermaid_flow` – flowchart parser
- `mermaid_sequence` – sequence diagram parser
- `mermaid_state` – state diagram parser
- `mermaid_packet` – packet diagram parser (bit/byte sizing helpers)

### Utilities

- `expr_lang` – expression language support types/errors
- `string_format` – rich template formatting and path-string support
- `common` / `mermaid_error` – shared parsing/file/debug/error utilities

## Dependencies

This crate depends on:

- Pest parser stack (`pest`, `pest_derive`, `pest_generator`)
- `core_utils` (workspace crate)
- `rust_macro` (workspace crate)
- `chrono`, `xml-rs`, `ansi_term`

## Quick start

Add as a dependency from the `CLR` workspace:

```toml
[dependencies]
parser_lib = { path = "parser_lib" }
```

### Parse JSON

```rust
use parser_lib::json::parse_json;

let input = r#"{"name":"core0","enabled":true}"#;
let obj = parse_json(input)?;
```

### Parse CSV and validate unique values in a column

```rust
use parser_lib::csv::{parse_csv, validate_csv_column_value_unique};

let csv = parse_csv("name,id\nA,1\nB,2\n")?;
let is_unique = validate_csv_column_value_unique(&csv, "id");
assert!(is_unique);
```

### Parse INI and flatten properties

```rust
use parser_lib::ini::{parse_ini_from_file, get_ini_properties};

let ini_file = parse_ini_from_file("config.ini")?;
let props = get_ini_properties(&ini_file);
```

### Parse Mermaid flowchart

```rust
use parser_lib::mermaid_flow::parse_flowchart;

let flow = r#"
flowchart TD
  A[Start] --> B[Run]
"#;

let program = parse_flowchart(flow)?;
```

### Parse formatted string templates

```rust
use core_utils::expr_value::ExprValue;
use parser_lib::string_format::formatted_string::FormattedString;

let fmt = FormattedString::parse(&"\"Value={0:X}\"".to_string())?;
let output = fmt.process(&vec![ExprValue::UInt32(Some(255))])?;
assert_eq!(output, "Value=FF");
```

## Build and test

From `CLR/`:

```bash
cargo check -p parser_lib
cargo test -p parser_lib
```

## Notes

- Public modules are exported in `src/lib.rs`.
- Most parsers expose `parse_*` functions for both string and file inputs.
- Error types are format-specific and returned via `Result<T, E>`.
