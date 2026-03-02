# `string_format` Module Notes

## Module Overview (`mod.rs`)

The `string_format` module lives in `CLR/parser_lib/src/string_format/` and is rooted at `mod.rs`.

- Declares submodules:
  - `string_formatter_item`
  - `formatted_string`
  - `path_string`
  - `datetime_format_string`
  - `string_format_error`
- Defines `StringFormatParser` (Pest parser) with grammars:
  - `basic_type_grammar.pest`
  - `string_format/string_format.pest`
- Provides shared parse helpers:
  - `from_pair_template`
  - `from_pair_vec_template`
  - `pair_to_i32`
- Provides numeric formatting entry points used by string interpolation:
  - `i64_to_string`
  - `u64_to_string`
  - `f64_to_string`

## String Format Cheat Sheet

### Top-level forms

- Raw string: `"..."`
- Verbatim string: `@"..."`
- Path string: `#"..."`, `#\"..."`, `#/"..."`

### Placeholders

Supported placeholder shapes:

- `{index}`
- `{index,width}`
- `{index:FmtPrec}`
- `{index,width:FmtPrec}`

Examples:

- `{0}`
- `{77,-4}`
- `{6,5:G3}`
- `{77:C89}`

### Formatter letters

- `d` / `D` → decimal (`Normal`)
- `h` / `H` → hex
- `o` / `O` → octal
- `b` / `B` → binary
- `e` / `E` → scientific
- `p` / `P` → percentage mode
- Any other char parses as `Other(char)`

### Escapes (raw string)

Supported escapes:

- `\\`
- `\"`
- `\{`
- `\}`
- `\n`
- `\r`
- `\t`
- `\0`

### Unicode escapes

- `\uXXXX`
- `\U00XXXXXX`

Examples from tests include `\u35F5` and `\U0001F61D`.

### Verbatim string behavior

- `@"..."` keeps backslashes as literal characters.
- `""` inside a verbatim string represents one double quote (`"`).

### Path string behavior

- `#"..."` defaults separator to `\`.
- `#\"..."` uses `\` as separator explicitly.
- `#/"..."` uses `/` as separator.

### Datetime format tokens

Supported tokens:

- Year: `yyyy`, `yy`
- Month: `MM`, `M`
- Day: `dd`, `d`
- Hour: `hh`, `h`
- Minute: `mm`, `m`
- Second: `ss`, `s`
- Millisecond: `msN`
- AM/PM: `am`, `pm` (case-insensitive in grammar)
- Any other chars are splitters

Examples:

- `yyyy-MM-dd hh:mm:ss:ms7 am`
- `yy-M-d^h:mm:s:ms3-pm`
- `yy-M-d^h:mm:s-pm`

## Parsing and processing flow

1. `FormattedString::parse` parses a top-level formatted string into AST.
2. AST variant is one of:
   - `FormattedString::Raw`
   - `FormattedString::Verbatim`
   - `FormattedString::Path`
3. `process(params)` resolves placeholder items (`StringFormatterItem`) against argument values.
4. Numeric values route through `i64_to_string` / `u64_to_string` / `f64_to_string`.

## Notes / caveats

- Percentage formatter (`p`/`P`) currently recurses with the same formatter and can loop indefinitely.
- Some formatter combinations fall into `todo!()` branches in conversion helpers.
- `DateTime::get_current_date_time()` uses simplified epoch math (not full calendar-accurate conversion).

## Key source files

- `CLR/parser_lib/src/string_format/mod.rs`
- `CLR/parser_lib/src/string_format/string_format.pest`
- `CLR/parser_lib/src/string_format/string_format_error.rs`
- `CLR/parser_lib/src/string_format/string_formatter_item.rs`
- `CLR/parser_lib/src/string_format/formatted_string.rs`
- `CLR/parser_lib/src/string_format/path_string.rs`
- `CLR/parser_lib/src/string_format/datetime_format_string.rs`
- `CLR/src/tests/string_format.rs`
