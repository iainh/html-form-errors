# Research: Developer Experience Improvements

**Date**: 2026-02-17
**Question**: What could we do to improve the developer experience of htmx-form-errors?
**Status**: Complete

## Context

The library is functional and well-documented. This research explores additional DX improvements.

## Findings

### 1. `Display` and `IntoResponse` Traits

`FormErrors` doesn't implement `Display` or `std::error::Error`. While the README correctly notes it's not a Rust error type, implementing `Display` (e.g. listing all errors) would help with logging/debugging. An optional `axum` feature with `IntoResponse` could streamline the handler pattern further.

### 2. `IntoIterator` / Iteration Support

There's no way to iterate over `(field, messages)` pairs. Adding `iter()`, `fields()`, `len()` (count of fields with errors), and `IntoIterator` would let users loop over errors generically — useful for custom rendering or serialization.

### 3. `From<(Option<&str>, String)>` / `FromIterator`

Building `FormErrors` from a vec of tuples or collecting from an iterator would be convenient:
```rust
let errors: FormErrors = vec![("email", "Bad"), ("name", "Short")].into();
```

### 4. Configurable CSS Classes

`invalid_class()` is hardcoded to Bootstrap's `"is-invalid"`. A `FormErrorsConfig` or a generic `invalid_class_name(field, valid, invalid)` method would support Tailwind, Bulma, etc.

### 5. `Deserialize` Support

Only `Serialize` is implemented. Adding `Deserialize` would allow round-tripping (useful for testing, API responses, or client-side error handling).

### 6. Improved Cargo.toml Metadata

Missing: `license`, `repository`, `keywords`, `categories`, `readme`, `authors`. These are needed for a proper crates.io publish.

### 7. Doc Tests and `#[doc = include_str!]` for README

Running the README examples as doc tests (via `doc = include_str!("../README.md")` or `cargo-readme`) would keep docs in sync with code.

### 8. `tracing` / `log` Integration

Optional feature to log when DB errors are parsed or fallback is used — helpful for debugging in production.

### 9. Field Name Mapping / Display Names

A common pain point: DB columns are `snake_case` but forms use human-readable labels. A field name mapper (e.g., `parser.field_alias("email_address", "email")`) would bridge this gap.

### 10. More Template Engine Macros

Only MiniJinja/Tera macros exist. Providing Askama helper snippets or a small Askama filter crate would round out the template story.

### 11. `merge()` as an Alias or Alternative to `extend()`

`extend` is the Rust convention, but `merge` reads better in application code. Could offer both.

### 12. Error Count / Summary Methods

Methods like `error_count()` (total messages across all fields), `field_count()` (number of fields with errors), and `summary()` → `"3 errors in 2 fields"` would help with logging and UI badges.

## Options by Priority

| Improvement | Impact | Effort |
|---|---|---|
| Iteration support (`iter`, `IntoIterator`, `len`) | High | Low |
| `Deserialize` | Medium | Very Low |
| Cargo.toml metadata for publish | High | Very Low |
| Configurable CSS class names | Medium | Low |
| Error count / summary methods | Medium | Very Low |
| Display trait | Medium | Low |
| Doc tests from README | Medium | Low |
| FromIterator / From<Vec> | Low | Very Low |
| Field name aliases | Medium | Medium |
| tracing feature | Low | Medium |
| axum IntoResponse feature | Medium | Medium |
| More template macros | Low | Medium |

## Recommendation

Start with the low-effort, high-impact items:
1. **Iteration + len** — unlocks generic rendering
2. **Cargo.toml metadata** — needed for crates.io
3. **Deserialize** — trivial with derive
4. **Error count methods** — useful for logging
5. **Configurable CSS classes** — broadens framework support beyond Bootstrap

## Open Questions

- Should `axum` integration live in this crate (feature-gated) or a separate `htmx-form-errors-axum` crate?
- Is crates.io publishing planned, or is git-only intentional?
- Should the template macros support Tailwind out of the box?
