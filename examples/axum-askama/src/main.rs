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
struct NumberForm {
    #[validate(range(max = 10, message = "Number must be 10 or less"))]
    number: i64,
}

#[derive(Template, WebTemplate)]
#[template(path = "form.html")]
struct FormPage {
    errors: FormErrors,
    number: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "_form.html")]
struct FormPartial {
    errors: FormErrors,
    number: String,
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
        number: String::new(),
    }
}

async fn handle_submit(Form(input): Form<NumberForm>) -> impl IntoResponse {
    let errors = input.form_errors();

    if errors.is_empty() {
        let html = format!(
            r#"<div class="alert alert-success" role="alert">
                 ✅ Success! You submitted: {}
               </div>"#,
            input.number
        );
        return Html(html).into_response();
    }

    FormPartial {
        errors,
        number: input.number.to_string(),
    }
    .into_response()
}
