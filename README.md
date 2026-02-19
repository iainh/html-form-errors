# htmx-form-errors

Field-level form validation errors for HTMX + server-side template rendering (MiniJinja, Tera, Askama, etc.).

Collects per-field error messages so your templates can display inline validation feedback using Bootstrap's `is-invalid` / `invalid-feedback` classes (or any similar pattern).

## Features

- `FormErrors` — a simple map of field names → error messages
- `serde` feature (default) — serializes `FormErrors` as a flat map for template contexts
- `validator` feature (default) — converts `validator::ValidationErrors` into `FormErrors`
- `DbErrorParser` — maps database error messages to user-friendly field errors
- `sqlx` feature (optional) — adds `DbErrorParser::parse(&sqlx::Error)` support
- Reusable **template macros** for MiniJinja / Tera (see `macros/form_errors.html`)

## Installation

```toml
[dependencies]
htmx-form-errors = { git = "https://github.com/iainh/htmx-form-errors" }
```

To also parse `sqlx::Error` directly:

```toml
[dependencies]
htmx-form-errors = { git = "https://github.com/iainh/htmx-form-errors", features = ["sqlx"] }
```

## Rust Usage

### Building errors manually

```rust
use htmx_form_errors::FormErrors;

let mut errors = FormErrors::new();
errors.add("email", "Email is required");
errors.add_base("Something went wrong");  // form-level error under "_base"

if errors.has_error("email") {
    println!("{}", errors.first("email").unwrap());
}
```

### Chainable builder

```rust
use htmx_form_errors::FormErrors;

let errors = FormErrors::new()
    .with_error("email", "Required")
    .with_error("name", "Too short");
```

### Helper methods

```rust
use htmx_form_errors::FormErrors;

let mut errors = FormErrors::new();

// add_opt: defaults to FormErrors::BASE ("_base") when field is None
errors.add_opt(Some("email"), "Bad email");
errors.add_opt(None, "General error");    // goes to "_base"

// invalid_class: returns "is-invalid" or "" (useful in Askama templates)
assert_eq!(errors.invalid_class("email"), "is-invalid");
assert_eq!(errors.invalid_class("name"), "");

// extend: merge another FormErrors into this one
let mut other = FormErrors::new();
other.add("age", "Must be positive");
errors.extend(other);
```

### From `validator::ValidationErrors`

The `ValidateExt` trait adds a one-liner `form_errors()` method to any type
implementing `validator::Validate`:

```rust,ignore
use htmx_form_errors::{FormErrors, ValidateExt};
use validator::Validate;

#[derive(Validate)]
struct SignupForm {
    #[validate(email(message = "Must be a valid email"))]
    email: String,
    #[validate(length(min = 3, message = "Name must be at least 3 characters"))]
    name: String,
}

fn handle_form(form: &SignupForm) -> FormErrors {
    form.form_errors()
}
```

### Mapping database errors with `DbErrorParser`

```rust
use htmx_form_errors::{DbErrorParser, DbErrorPattern, FormErrors};

let parser = DbErrorParser::new()
    .rule(DbErrorPattern::Unique, None, "A record with this value already exists")
    .rule_for_field("email", DbErrorPattern::Unique, "This email is already taken")
    .rule_for_field("subnet", DbErrorPattern::InvalidInputSyntax, "Invalid subnet/CIDR format")
    .fallback("An unexpected error occurred");

// One-liner: parse a DB error message and add it to errors directly
let mut errors = FormErrors::new();
parser.add_message_to(&mut errors, "violates unique constraint on email");
assert_eq!(errors.first("email"), Some("This email is already taken"));

// With the `sqlx` feature, parse a &sqlx::Error directly:
// parser.add_sqlx_error_to(&mut errors, &sqlx_error);

// Or use the lower-level API if you need the raw (field, message) pair:
// let (field, message) = parser.parse_message("...");
// let (field, message) = parser.parse(&sqlx_error);  // sqlx feature
```

### Handler pattern (Axum example)

With HTMX, both success and validation failure return **rendered HTML** — the
handler always returns an HTTP response, not a `Result<_, FormErrors>`.
`FormErrors` is part of the template context, not a Rust error type:

```rust,ignore
async fn handle_submit(
    State(state): State<AppState>,
    Form(input): Form<MyForm>,
) -> impl IntoResponse {
    // 1. Validate the input
    let mut errors = input.form_errors();

    // 2. If validation passed, try the DB operation
    if errors.is_empty() {
        match state.db.insert(&input).await {
            Ok(_) => {
                // Success — redirect or return a success fragment
                return Redirect::to("/success").into_response();
            }
            Err(e) => state.parser.add_sqlx_error_to(&mut errors, &e),
        }
    }

    // 3. Re-render the form with errors (HTMX swaps this into the page)
    let html = state.templates.render("form.html", context! {
        errors => &errors,
        value => &input.value,
    });
    Html(html).into_response()
}
```

The key insight: the handler **always succeeds** from HTTP's perspective. It
either returns a redirect/success fragment, or the re-rendered form with inline
error messages. HTMX swaps the response in either case.

### Passing errors to a template context

With the `serde` feature (enabled by default), `FormErrors` serializes as a
flat `{ "field": ["msg", ...] }` map — ready for MiniJinja, Tera, or any
serde-based template engine:

```rust,ignore
// MiniJinja
let ctx = minijinja::context! { errors => &errors, email => &form.email };

// Tera
let mut ctx = tera::Context::new();
ctx.insert("errors", &errors);
```

## HTML / HTMX Usage

### Template macros (MiniJinja / Tera)

Copy `macros/form_errors.html` into your template directory. It provides
these macros:

#### Field-group macros (recommended)

These render a complete Bootstrap field group (label + input + error feedback)
in a single call:

| Macro | Purpose |
|---|---|
| `input_group(errors, field, label, value="", type="text", base="form-control")` | Full input field with label and error feedback |
| `select_group(errors, field, label, options, selected="", base="form-select")` | Full select field with label and error feedback |
| `textarea_group(errors, field, label, value="", rows=3, base="form-control")` | Full textarea with label and error feedback |
| `checkbox_group(errors, field, label, checked=false, value="1")` | Checkbox with label and error feedback |
| `radio_group(errors, field, label, options, selected="")` | Radio button group with label and error feedback |
| `switch_group(errors, field, label, checked=false, value="1")` | Toggle switch with label and error feedback |
| `floating_input_group(errors, field, label, value="", type="text")` | Floating-label input field |
| `floating_select_group(errors, field, label, options, selected="")` | Floating-label select field |
| `floating_textarea_group(errors, field, label, value="", height="100px")` | Floating-label textarea |
| `base_errors(errors, field="_base")` | Renders form-level errors as a Bootstrap alert |

```jinja
{% from "macros/form_errors.html" import input_group, checkbox_group, radio_group, floating_input_group, base_errors %}

<form hx-post="/signup" hx-target="#signup-form" hx-swap="outerHTML" id="signup-form">
  {{ base_errors(errors=errors) }}
  {{ input_group(errors=errors, field='email', label='Email', value=email, type='email') }}
  {{ input_group(errors=errors, field='name', label='Name', value=name) }}
  {{ radio_group(errors=errors, field='role', label='Role', options=roles, selected=role) }}
  {{ checkbox_group(errors=errors, field='agree', label='I agree to the terms', checked=agree) }}
  <button type="submit" class="btn btn-primary">Sign Up</button>
</form>
```

#### Low-level macros

For custom layouts, the primitive macros are also available:

| Macro | Purpose |
|---|---|
| `input_class(errors, field, base="form-control")` | Returns `"form-control is-invalid"` or `"form-control"` |
| `select_class(errors, field, base="form-select")` | Same, but defaults to `"form-select"` |
| `check_class(errors, field, base="form-check-input")` | Same, but defaults to `"form-check-input"` (for checkboxes/radios/switches) |
| `first_error(errors, field)` | Renders `<div class="invalid-feedback">` with the first error |
| `all_errors(errors, field)` | Renders all errors for a field |

```jinja
{% from "macros/form_errors.html" import input_class, first_error, base_errors %}

<div class="mb-3">
  <label for="email" class="form-label">Email</label>
  <input type="email" name="email" id="email"
         class="{{ input_class(errors=errors, field='email') }}"
         value="{{ email | default('') }}">
  {{ first_error(errors=errors, field='email') }}
</div>
```

### Form setup

- `hx-post` — submits the form via HTMX to your endpoint.
- `hx-target` / `hx-swap="outerHTML"` — replaces the entire form with the
  server's response (the re-rendered form, now with error classes).

### Manual template approach (without macros)

If you prefer not to use the macro file, the equivalent hand-written markup is:

```html
<input
  type="email"
  name="email"
  class="form-control {% if errors.email is defined %}is-invalid{% endif %}"
  value="{{ email | default('') }}"
>
{% if errors.email is defined %}
  <div class="invalid-feedback">{{ errors.email[0] }}</div>
{% endif %}
```

### Askama templates

Enable the `askama` feature for first-class integration:

```toml
[dependencies]
htmx-form-errors = { git = "https://github.com/iainh/htmx-form-errors", features = ["askama"] }
```

Askama can call Rust methods directly on the struct. `feedback_html()` and
`base_errors_html()` return [`SafeHtml`], which implements Askama's `HtmlSafe`
trait — no `|safe` filter needed:

```html
{{ errors.base_errors_html() }}

<input type="email" name="email"
       class="form-control {{ errors.invalid_class("email") }}"
       value="{{ email }}">
{{ errors.feedback_html("email") }}

<div class="form-check">
  <input type="checkbox" name="agree" class="{{ errors.check_class("agree") }}">
  <label class="form-check-label">I agree to the terms</label>
  {{ errors.feedback_html("agree") }}
</div>
```

The HTML helpers escape error messages internally before wrapping in `SafeHtml`.

## How It Fits Together

1. User submits the form via HTMX (`hx-post`).
2. Server validates with `validator` → converts errors to `FormErrors`.
3. If a database insert/update fails, `DbErrorParser` maps the DB error to a
   field-level message via `add_message_to` / `add_sqlx_error_to`.
4. Server re-renders the same form template with `FormErrors` in context.
5. HTMX swaps the response into the page — fields now show `is-invalid` and
   feedback messages inline.

## Example

See `examples/axum-minijinja/` for a complete working example with Axum, MiniJinja,
and HTMX — a single form that validates a number is 10 or less.

Run it with:

```sh
cd examples/axum-minijinja
cargo run
# Open http://localhost:3000
```

## Licence

MIT
