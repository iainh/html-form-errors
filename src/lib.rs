#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct _ReadmeDocTests;

use std::collections::HashMap;
use std::fmt;

/// A pre-escaped HTML string that is safe to render without further escaping.
///
/// With the `askama` feature enabled, this type implements
/// [`askama::filters::HtmlSafe`], so Askama will render it without
/// double-escaping — no `|safe` filter needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeHtml(String);

impl SafeHtml {
    /// Returns the inner HTML string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SafeHtml {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(feature = "askama")]
impl askama::filters::HtmlSafe for SafeHtml {}

/// Field-level validation errors for template rendering.
///
/// Collects per-field error messages that can be queried in templates
/// to display inline validation feedback (e.g., Bootstrap's `is-invalid` class).
///
/// # Example
///
/// ```
/// use htmx_form_errors::FormErrors;
///
/// let mut errors = FormErrors::new();
/// errors.add("email", "Email is required");
/// assert!(errors.has_error("email"));
/// assert_eq!(errors.first("email"), Some("Email is required"));
/// assert!(!errors.has_error("name"));
/// ```
#[derive(Debug, Default, Clone)]
pub struct FormErrors {
    errors: HashMap<String, Vec<String>>,
}

/// Key used for form-level (non-field-specific) errors.
impl FormErrors {
    pub const BASE: &'static str = "_base";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Add an error message for a field.
    pub fn add(&mut self, field: &str, message: &str) {
        self.errors
            .entry(field.to_string())
            .or_default()
            .push(message.to_string());
    }

    /// Add a form-level (non-field-specific) error message.
    pub fn add_base(&mut self, message: &str) {
        self.add(Self::BASE, message);
    }

    /// Add an error for a field, defaulting to [`BASE`](Self::BASE) when `field` is `None`.
    pub fn add_opt(&mut self, field: Option<&str>, message: &str) {
        self.add(field.unwrap_or(Self::BASE), message);
    }

    /// Chainable builder: add an error and return `self`.
    pub fn with_error(mut self, field: &str, message: &str) -> Self {
        self.add(field, message);
        self
    }

    /// Merge all errors from `other` into `self`.
    pub fn extend(&mut self, other: FormErrors) {
        for (k, vs) in other.errors {
            self.errors.entry(k).or_default().extend(vs);
        }
    }

    /// Get all error messages for a field.
    pub fn get(&self, field: &str) -> Option<&Vec<String>> {
        self.errors.get(field)
    }

    /// Check if a field has errors.
    pub fn has_error(&self, field: &str) -> bool {
        self.errors.contains_key(field)
    }

    /// Get the first error message for a field.
    pub fn first(&self, field: &str) -> Option<&str> {
        self.errors
            .get(field)
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }

    /// Returns `"is-invalid"` if the field has errors, otherwise `""`.
    pub fn invalid_class(&self, field: &str) -> &'static str {
        if self.has_error(field) {
            "is-invalid"
        } else {
            ""
        }
    }

    /// Returns `"form-check-input is-invalid"` if the field has errors,
    /// otherwise `"form-check-input"`.
    ///
    /// Useful in Askama templates for checkboxes, radios, and switches:
    ///
    /// ```html,ignore
    /// <input type="checkbox" class="{{ errors.check_class("agree") }}">
    /// ```
    pub fn check_class(&self, field: &str) -> &'static str {
        if self.has_error(field) {
            "form-check-input is-invalid"
        } else {
            "form-check-input"
        }
    }

    /// Returns the first error for a field wrapped in Bootstrap invalid-feedback
    /// markup, or an empty string if no errors.
    ///
    /// The error message is HTML-escaped internally. With the `askama` feature,
    /// the returned [`SafeHtml`] implements `HtmlSafe` so no `|safe` filter is
    /// needed:
    ///
    /// ```html,ignore
    /// {{ errors.feedback_html("email") }}
    /// ```
    pub fn feedback_html(&self, field: &str) -> SafeHtml {
        match self.first(field) {
            Some(msg) => SafeHtml(format!(
                r#"<div class="invalid-feedback">{}</div>"#,
                html_escape(msg)
            )),
            None => SafeHtml(String::new()),
        }
    }

    /// Returns all base-level errors as a Bootstrap alert, or an empty string
    /// if there are none.
    ///
    /// The error messages are HTML-escaped internally. With the `askama` feature,
    /// the returned [`SafeHtml`] implements `HtmlSafe` so no `|safe` filter is
    /// needed:
    ///
    /// ```html,ignore
    /// {{ errors.base_errors_html() }}
    /// ```
    pub fn base_errors_html(&self) -> SafeHtml {
        match self.get(Self::BASE) {
            Some(messages) => {
                let mut html = String::from(r#"<div class="alert alert-danger">"#);
                for msg in messages {
                    html.push_str(&format!("<div>{}</div>", html_escape(msg)));
                }
                html.push_str("</div>");
                SafeHtml(html)
            }
            None => SafeHtml(String::new()),
        }
    }
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(feature = "serde")]
impl serde::Serialize for FormErrors {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.errors.serialize(serializer)
    }
}

/// Extension trait that adds a one-liner `form_errors()` method to any
/// type implementing [`validator::Validate`].
///
/// # Example
///
/// ```ignore
/// use htmx_form_errors::ValidateExt;
/// use validator::Validate;
///
/// #[derive(Validate)]
/// struct MyForm {
///     #[validate(email(message = "Must be a valid email"))]
///     email: String,
/// }
///
/// let form = MyForm { email: "bad".into() };
/// let errors = form.form_errors();
/// assert!(errors.has_error("email"));
/// ```
#[cfg(feature = "validator")]
pub trait ValidateExt: validator::Validate {
    /// Validate `self` and return any validation errors as [`FormErrors`].
    ///
    /// Returns an empty `FormErrors` when validation passes.
    fn form_errors(&self) -> FormErrors {
        match self.validate() {
            Ok(()) => FormErrors::new(),
            Err(e) => FormErrors::from(e),
        }
    }
}

#[cfg(feature = "validator")]
impl<T: validator::Validate> ValidateExt for T {}

#[cfg(feature = "validator")]
impl From<validator::ValidationErrors> for FormErrors {
    fn from(errors: validator::ValidationErrors) -> Self {
        let mut form_errors = FormErrors::new();
        for (field, field_errors) in errors.field_errors() {
            for error in field_errors {
                let message = error
                    .message
                    .as_ref()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| format!("Invalid value for {}", field));
                form_errors.add(&field, &message);
            }
        }
        form_errors
    }
}

/// Common database error patterns for matching against error messages.
#[derive(Debug, Clone)]
pub enum DbErrorPattern {
    /// Matches "violates foreign key constraint"
    ForeignKey,
    /// Matches "violates unique constraint"
    Unique,
    /// Matches "violates check constraint"
    Check,
    /// Matches "invalid input syntax for type"
    InvalidInputSyntax,
    /// Matches "invalid input value for enum"
    InvalidEnumValue,
    /// Matches an arbitrary substring in the error message.
    Custom(String),
}

impl DbErrorPattern {
    fn as_str(&self) -> &str {
        match self {
            Self::ForeignKey => "violates foreign key constraint",
            Self::Unique => "violates unique constraint",
            Self::Check => "violates check constraint",
            Self::InvalidInputSyntax => "invalid input syntax for type",
            Self::InvalidEnumValue => "invalid input value for enum",
            Self::Custom(s) => s.as_str(),
        }
    }
}

/// A rule for mapping database error messages to user-friendly field errors.
///
/// When a database error message matches `pattern`, the error is mapped
/// to the given `field` with the given `message`.
#[derive(Debug, Clone)]
pub struct DbErrorRule {
    pub pattern: DbErrorPattern,
    pub field: Option<String>,
    pub message: String,
}

/// Configurable parser that maps database error messages to [`FormErrors`].
///
/// Register rules that match substrings in database error messages, then call
/// [`parse`](DbErrorParser::parse) to convert a database error into a
/// `(Option<field>, message)` pair suitable for adding to a [`FormErrors`].
///
/// # Example
///
/// ```
/// use htmx_form_errors::{DbErrorParser, DbErrorPattern};
///
/// let parser = DbErrorParser::new()
///     .rule(DbErrorPattern::Unique, None, "A record with this value already exists")
///     .rule_for_field("pcid", DbErrorPattern::Unique, "This PCID already exists")
///     .rule_for_field("subnet", DbErrorPattern::InvalidInputSyntax, "Invalid subnet/CIDR format")
///     .fallback("An error occurred");
///
/// // When no sqlx feature, you can still use parse_message directly:
/// let (field, msg) = parser.parse_message("violates unique constraint on pcid");
/// assert_eq!(field, Some("pcid"));
/// assert_eq!(msg, "This PCID already exists");
/// ```
#[derive(Debug, Clone)]
pub struct DbErrorParser {
    rules: Vec<DbErrorRule>,
    fallback_message: String,
}

impl DbErrorParser {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            fallback_message: "An error occurred".to_string(),
        }
    }

    /// Add a rule that matches when the error message contains `pattern`.
    /// `field` is the form field to associate the error with (or `None` for a general error).
    pub fn rule(mut self, pattern: DbErrorPattern, field: Option<&str>, message: &str) -> Self {
        self.rules.push(DbErrorRule {
            pattern,
            field: field.map(|f| f.to_string()),
            message: message.to_string(),
        });
        self
    }

    /// Shorthand: add a rule that matches when the error contains both `pattern` and `field_name`.
    pub fn rule_for_field(
        mut self,
        field_name: &str,
        pattern: DbErrorPattern,
        message: &str,
    ) -> Self {
        self.rules.push(DbErrorRule {
            pattern,
            field: Some(field_name.to_string()),
            message: message.to_string(),
        });
        self
    }

    /// Set the fallback message when no rules match.
    pub fn fallback(mut self, message: &str) -> Self {
        self.fallback_message = message.to_string();
        self
    }

    /// Parse a raw database error message string into a `(field, message)` pair.
    ///
    /// Rules with a `field` set are checked first: the message must contain both
    /// the rule's `pattern` and the field name. Then field-less rules are checked.
    pub fn parse_message(&self, db_message: &str) -> (Option<&str>, String) {
        // First pass: rules that have a field — match pattern AND field name in message
        for rule in &self.rules {
            if let Some(ref field) = rule.field {
                if db_message.contains(rule.pattern.as_str()) && db_message.contains(field.as_str())
                {
                    return (Some(field.as_str()), rule.message.clone());
                }
            }
        }

        // Second pass: rules without a field — match pattern only
        for rule in &self.rules {
            if rule.field.is_none() && db_message.contains(rule.pattern.as_str()) {
                return (None, rule.message.clone());
            }
        }

        (None, self.fallback_message.clone())
    }

    /// Parse a raw database error message and add the result to `errors`.
    pub fn add_message_to(&self, errors: &mut FormErrors, db_message: &str) {
        let (field, msg) = self.parse_message(db_message);
        errors.add_opt(field, &msg);
    }

    /// Parse a [`sqlx::Error`] into a `(field, message)` pair.
    ///
    /// Only database errors are parsed; other sqlx errors return the fallback.
    #[cfg(feature = "sqlx")]
    pub fn parse(&self, error: &sqlx::Error) -> (Option<&str>, String) {
        if let sqlx::Error::Database(ref db_err) = error {
            self.parse_message(db_err.message())
        } else {
            (None, self.fallback_message.clone())
        }
    }

    /// Parse a [`sqlx::Error`] and add the result directly to `errors`.
    #[cfg(feature = "sqlx")]
    pub fn add_sqlx_error_to(&self, errors: &mut FormErrors, error: &sqlx::Error) {
        let (field, msg) = self.parse(error);
        errors.add_opt(field, &msg);
    }
}

impl Default for DbErrorParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_errors() {
        let errors = FormErrors::new();
        assert!(errors.is_empty());
        assert!(!errors.has_error("field"));
        assert_eq!(errors.first("field"), None);
        assert_eq!(errors.get("field"), None);
    }

    #[test]
    fn add_and_query() {
        let mut errors = FormErrors::new();
        errors.add("email", "Email is required");
        errors.add("email", "Email is invalid");
        errors.add("name", "Name is required");

        assert!(!errors.is_empty());
        assert!(errors.has_error("email"));
        assert_eq!(errors.first("email"), Some("Email is required"));
        assert_eq!(errors.get("email").unwrap().len(), 2);
        assert!(!errors.has_error("age"));
    }

    #[test]
    fn parser_field_specific_rules() {
        let parser = DbErrorParser::new()
            .rule(DbErrorPattern::Unique, None, "Duplicate value")
            .rule_for_field("pcid", DbErrorPattern::Unique, "This PCID already exists")
            .fallback("Something went wrong");

        let (field, msg) = parser.parse_message("violates unique constraint on pcid");
        assert_eq!(field, Some("pcid"));
        assert_eq!(msg, "This PCID already exists");
    }

    #[test]
    fn parser_generic_rule() {
        let parser = DbErrorParser::new()
            .rule(DbErrorPattern::Unique, None, "Duplicate value")
            .rule_for_field("pcid", DbErrorPattern::Unique, "This PCID already exists")
            .fallback("Something went wrong");

        let (field, msg) = parser.parse_message("violates unique constraint on email");
        assert_eq!(field, None);
        assert_eq!(msg, "Duplicate value");
    }

    #[test]
    fn parser_fallback() {
        let parser = DbErrorParser::new()
            .rule(DbErrorPattern::Unique, None, "Duplicate value")
            .fallback("Something went wrong");

        let (field, msg) = parser.parse_message("connection refused");
        assert_eq!(field, None);
        assert_eq!(msg, "Something went wrong");
    }

    #[test]
    fn parser_custom_pattern() {
        let parser = DbErrorParser::new()
            .rule_for_field(
                "subnet",
                DbErrorPattern::Custom("invalid cidr".into()),
                "Bad CIDR",
            )
            .fallback("Unknown error");

        let (field, msg) = parser.parse_message("invalid cidr value for subnet");
        assert_eq!(field, Some("subnet"));
        assert_eq!(msg, "Bad CIDR");
    }

    #[test]
    fn add_base() {
        let mut errors = FormErrors::new();
        errors.add_base("Something went wrong");
        assert!(errors.has_error(FormErrors::BASE));
        assert_eq!(errors.first(FormErrors::BASE), Some("Something went wrong"));
    }

    #[test]
    fn add_opt_with_field() {
        let mut errors = FormErrors::new();
        errors.add_opt(Some("email"), "Bad email");
        assert!(errors.has_error("email"));
        assert!(!errors.has_error(FormErrors::BASE));
    }

    #[test]
    fn add_opt_without_field() {
        let mut errors = FormErrors::new();
        errors.add_opt(None, "General error");
        assert!(errors.has_error(FormErrors::BASE));
        assert_eq!(errors.first(FormErrors::BASE), Some("General error"));
    }

    #[test]
    fn with_error_chaining() {
        let errors = FormErrors::new()
            .with_error("email", "Required")
            .with_error("name", "Too short");
        assert!(errors.has_error("email"));
        assert!(errors.has_error("name"));
        assert_eq!(errors.first("email"), Some("Required"));
    }

    #[test]
    fn extend_errors() {
        let mut a = FormErrors::new();
        a.add("email", "Required");

        let mut b = FormErrors::new();
        b.add("email", "Invalid format");
        b.add("name", "Too short");

        a.extend(b);
        assert_eq!(a.get("email").unwrap().len(), 2);
        assert!(a.has_error("name"));
    }

    #[test]
    fn invalid_class() {
        let errors = FormErrors::new().with_error("email", "Bad");
        assert_eq!(errors.invalid_class("email"), "is-invalid");
        assert_eq!(errors.invalid_class("name"), "");
    }

    #[test]
    fn check_class() {
        let errors = FormErrors::new().with_error("agree", "You must agree");
        assert_eq!(errors.check_class("agree"), "form-check-input is-invalid");
        assert_eq!(errors.check_class("other"), "form-check-input");
    }

    #[test]
    fn feedback_html_with_error() {
        let errors = FormErrors::new().with_error("email", "Required");
        assert_eq!(
            errors.feedback_html("email").as_str(),
            r#"<div class="invalid-feedback">Required</div>"#
        );
    }

    #[test]
    fn feedback_html_without_error() {
        let errors = FormErrors::new();
        assert_eq!(errors.feedback_html("email").as_str(), "");
    }

    #[test]
    fn feedback_html_escapes_html() {
        let errors = FormErrors::new().with_error("email", "<script>alert('xss')</script>");
        assert_eq!(
            errors.feedback_html("email").as_str(),
            r#"<div class="invalid-feedback">&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;</div>"#
        );
    }

    #[test]
    fn base_errors_html_with_errors() {
        let mut errors = FormErrors::new();
        errors.add_base("Error one");
        errors.add_base("Error two");
        assert_eq!(
            errors.base_errors_html().as_str(),
            r#"<div class="alert alert-danger"><div>Error one</div><div>Error two</div></div>"#
        );
    }

    #[test]
    fn base_errors_html_without_errors() {
        let errors = FormErrors::new();
        assert_eq!(errors.base_errors_html().as_str(), "");
    }

    #[test]
    fn add_message_to() {
        let parser = DbErrorParser::new()
            .rule_for_field("email", DbErrorPattern::Unique, "Already taken")
            .fallback("Unknown error");

        let mut errors = FormErrors::new();
        parser.add_message_to(&mut errors, "violates unique constraint on email");
        assert_eq!(errors.first("email"), Some("Already taken"));
    }

    #[test]
    fn add_message_to_fallback() {
        let parser = DbErrorParser::new().fallback("Unknown error");

        let mut errors = FormErrors::new();
        parser.add_message_to(&mut errors, "something unexpected");
        assert_eq!(errors.first(FormErrors::BASE), Some("Unknown error"));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn serde_serializes_as_flat_map() {
        let errors = FormErrors::new()
            .with_error("email", "Required")
            .with_error("email", "Invalid")
            .with_error("name", "Too short");

        let json = serde_json::to_value(&errors).unwrap();
        assert!(json.is_object());
        assert_eq!(json["email"][0], "Required");
        assert_eq!(json["email"][1], "Invalid");
        assert_eq!(json["name"][0], "Too short");
    }
}
