use axum::{
    extract::State,
    response::{Html, IntoResponse},
    routing::get,
    Form, Router,
};
use htmx_form_errors::FormErrors;
use minijinja::{context, Environment};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
struct NumberForm {
    #[validate(range(max = 10, message = "Number must be 10 or less"))]
    number: i64,
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
    render_page(&state, &FormErrors::new(), "")
}

async fn handle_submit(
    State(state): State<Arc<AppState>>,
    Form(input): Form<NumberForm>,
) -> impl IntoResponse {
    let errors = match input.validate() {
        Ok(_) => FormErrors::new(),
        Err(e) => FormErrors::from(e),
    };

    if errors.is_empty() {
        // Success — return a success message that HTMX swaps in
        let html = format!(
            r#"<div class="alert alert-success" role="alert">
                 ✅ Success! You submitted: {}
               </div>"#,
            input.number
        );
        return Html(html).into_response();
    }

    // Re-render just the form partial with errors
    render_partial(&state, &errors, &input.number.to_string()).into_response()
}

fn render_page(state: &AppState, errors: &FormErrors, number: &str) -> Html<String> {
    let tmpl = state.env.get_template("form.html").unwrap();
    let html = tmpl
        .render(context! {
            errors => errors,
            number => number,
        })
        .unwrap();
    Html(html)
}

fn render_partial(state: &AppState, errors: &FormErrors, number: &str) -> Html<String> {
    let tmpl = state.env.get_template("_form.html").unwrap();
    let html = tmpl
        .render(context! {
            errors => errors,
            number => number,
        })
        .unwrap();
    Html(html)
}
