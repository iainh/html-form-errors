use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Form, Router,
};
use htmx_form_errors::{FormErrors, ValidateExt};
use minijinja::{context, Environment};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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

#[derive(Serialize)]
struct SelectOption {
    value: String,
    label: String,
}

fn role_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "".into(), label: "Select a role…".into() },
        SelectOption { value: "admin".into(), label: "Admin".into() },
        SelectOption { value: "editor".into(), label: "Editor".into() },
        SelectOption { value: "viewer".into(), label: "Viewer".into() },
    ]
}

fn contact_method_options() -> Vec<SelectOption> {
    vec![
        SelectOption { value: "email".into(), label: "Email".into() },
        SelectOption { value: "phone".into(), label: "Phone".into() },
        SelectOption { value: "sms".into(), label: "SMS".into() },
    ]
}

struct AppState {
    env: Environment<'static>,
}

#[tokio::main]
async fn main() {
    let mut env = Environment::new();
    env.set_loader(minijinja::path_loader("templates"));

    let state = Arc::new(AppState { env });

    let app = Router::new()
        .route("/", get(show_form).post(handle_submit))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("Listening on http://localhost:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn show_form(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    render_page(&state, &FormErrors::new(), &ContactForm {
        name: String::new(),
        email: String::new(),
        role: String::new(),
        bio: String::new(),
        agree: None,
        contact_method: String::new(),
        notifications: None,
    })
}

async fn handle_submit(
    State(state): State<Arc<AppState>>,
    Form(input): Form<ContactForm>,
) -> impl IntoResponse {
    let mut errors = input.form_errors();

    if input.agree.is_none() {
        errors.add("agree", "You must agree to the terms");
    }

    if errors.is_empty() {
        let html = format!(
            r#"<div class="alert alert-success" role="alert">
                 ✅ Success! Contact form submitted for {} ({})
               </div>"#,
            input.name, input.email
        );
        return Html(html).into_response();
    }

    render_partial(&state, &errors, &input).into_response()
}

fn render_page(state: &AppState, errors: &FormErrors, input: &ContactForm) -> Html<String> {
    let tmpl = state.env.get_template("form.html").unwrap();
    let html = tmpl
        .render(context! {
            errors => errors,
            name => input.name,
            email => input.email,
            role => input.role,
            bio => input.bio,
            agree => input.agree,
            contact_method => input.contact_method,
            notifications => input.notifications,
            role_options => role_options(),
            contact_method_options => contact_method_options(),
        })
        .unwrap();
    Html(html)
}

fn render_partial(state: &AppState, errors: &FormErrors, input: &ContactForm) -> Html<String> {
    let tmpl = state.env.get_template("_form.html").unwrap();
    let html = tmpl
        .render(context! {
            errors => errors,
            name => input.name,
            email => input.email,
            role => input.role,
            bio => input.bio,
            agree => input.agree,
            contact_method => input.contact_method,
            notifications => input.notifications,
            role_options => role_options(),
            contact_method_options => contact_method_options(),
        })
        .unwrap();
    Html(html)
}
