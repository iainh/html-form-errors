use askama::Template;
use askama_web::WebTemplate;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Form, Router,
};
use htmx_form_errors::{FormErrors, ValidateExt};
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
struct ContactForm {
    #[validate(length(min = 2, message = "Name must be at least 2 characters"))]
    name: String,
    #[validate(email(message = "Must be a valid email"))]
    email: String,
    #[validate(length(min = 1, message = "Please select a role"))]
    role: String,
    bio: String,
    #[serde(default)]
    agree: Option<String>,
    #[validate(length(min = 1, message = "Please select a contact method"))]
    contact_method: String,
    #[serde(default)]
    notifications: Option<String>,
}

struct SelectOption {
    value: String,
    label: String,
}

fn role_options() -> Vec<SelectOption> {
    vec![
        SelectOption {
            value: "developer".into(),
            label: "Developer".into(),
        },
        SelectOption {
            value: "designer".into(),
            label: "Designer".into(),
        },
        SelectOption {
            value: "manager".into(),
            label: "Manager".into(),
        },
        SelectOption {
            value: "other".into(),
            label: "Other".into(),
        },
    ]
}

fn contact_method_options() -> Vec<SelectOption> {
    vec![
        SelectOption {
            value: "email".into(),
            label: "Email".into(),
        },
        SelectOption {
            value: "phone".into(),
            label: "Phone".into(),
        },
        SelectOption {
            value: "sms".into(),
            label: "SMS".into(),
        },
    ]
}

#[derive(Template, WebTemplate)]
#[template(path = "form.html")]
struct FormPage {
    errors: FormErrors,
    name: String,
    email: String,
    role: String,
    bio: String,
    agree: Option<String>,
    contact_method: String,
    notifications: Option<String>,
    role_options: Vec<SelectOption>,
    contact_method_options: Vec<SelectOption>,
}

#[derive(Template, WebTemplate)]
#[template(path = "_form.html")]
struct FormPartial {
    errors: FormErrors,
    name: String,
    email: String,
    role: String,
    bio: String,
    agree: Option<String>,
    contact_method: String,
    notifications: Option<String>,
    role_options: Vec<SelectOption>,
    contact_method_options: Vec<SelectOption>,
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(show_form).post(handle_submit));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn show_form() -> FormPage {
    FormPage {
        errors: FormErrors::new(),
        name: String::new(),
        email: String::new(),
        role: String::new(),
        bio: String::new(),
        agree: None,
        contact_method: String::new(),
        notifications: None,
        role_options: role_options(),
        contact_method_options: contact_method_options(),
    }
}

async fn handle_submit(Form(input): Form<ContactForm>) -> impl IntoResponse {
    let mut errors = input.form_errors();

    if input.agree.is_none() {
        errors.add("agree", "You must agree to the terms");
    }

    if errors.is_empty() {
        let html = format!(
            r#"<div class="alert alert-success" role="alert">
                 ✅ Success! Submitted contact form for {} ({})
               </div>"#,
            input.name, input.email
        );
        return Html(html).into_response();
    }

    FormPartial {
        errors,
        name: input.name,
        email: input.email,
        role: input.role,
        bio: input.bio,
        agree: input.agree,
        contact_method: input.contact_method,
        notifications: input.notifications,
        role_options: role_options(),
        contact_method_options: contact_method_options(),
    }
    .into_response()
}
