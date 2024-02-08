use std::time::Duration;

use askama_axum::Template;
use axum::extract::{Path, State};

// use tower_http_static::FileDir;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{serve, Form, Router};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower_http::services::ServeFile;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let db_connection_str = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:mysecretpassword@172.24.64.1".to_string());

    // set up connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await
        .expect("can't connect to database");

    let assets_path = std::env::current_dir().unwrap();

    let app = Router::new()
        .route("/", get(index))
        .route("/:id", get(redirect))
        .route("/short", post(shorten))
        .route("/long", post(longen))
        .nest_service(
            "/templates",
            ServeFile::new(format!("{}/templates/output.css", assets_path.to_str().unwrap())),
        )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct FormBaseTemplate {
    title: String,
}

async fn index() -> impl IntoResponse {
    let template = FormBaseTemplate {
        title: String::from("hola mundo"),
    };
    template
}

#[derive(serde::Deserialize)]
struct UrlId {
    id: String,
}

#[derive(serde::Deserialize, sqlx::FromRow, Debug)]
struct DBFields {
    url: String,
}

async fn redirect(
    State(pool): State<PgPool>,
    Path(url_id): Path<UrlId>,
) -> Result<Redirect, StatusCode> {
    println!("helloz");
    let url = match sqlx::query_as::<_, DBFields>("SELECT url FROM urls WHERE id = $1")
        .bind(url_id.id)
        .fetch_one(&pool)
        .await
    {
        Ok(url) => url,
        Err(e) => {
            println!("Error fetching URL from the database: {:?}", e);
            return Err(match e {
                sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            });
        }
    };
    println!("result={:?}", url);
    Ok(Redirect::to(&url.url))
}

#[derive(serde::Deserialize)]
struct ShortenedUrl {
    url: String,
}

#[derive(Template)]
#[template(path = "redirect.html")]
struct RedirectTemplate {
    redirected_url: String,
}

async fn shorten(
    State(pool): State<PgPool>,
    Form(shortened_url): Form<ShortenedUrl>,
) -> Result<RedirectTemplate, StatusCode> {
    let id = &nanoid::nanoid!(6);
    println!("id={:?}", id);

    sqlx::query("INSERT INTO urls(id, url) VALUES ($1, $2)")
        .bind(id)
        .bind(shortened_url.url)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let redirected_template = RedirectTemplate {
        // title: String::from("Redirecting..."),
        redirected_url: format!("http://127.0.0.1:8000/{}", id),
    };

    Ok(redirected_template)
}

async fn longen(
    State(pool): State<PgPool>,
    Form(shortened_url): Form<ShortenedUrl>,
) -> Result<RedirectTemplate, StatusCode> {
    let id = &nanoid::nanoid!(420);
    println!("ideez={:?}", id);

    sqlx::query("INSERT INTO urls(id, url) VALUES ($1, $2)")
        .bind(id)
        .bind(shortened_url.url)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let redirected_template = RedirectTemplate {
        // title: String::from("Redirecting..."),
        redirected_url: format!("http://127.0.0.1:8000/{}", id),
    };

    Ok(redirected_template)

    // Ok(format!("https://url-shortener-relatively/{}", id))
    // Ok(format!("http://127.0.0.1:8000/{}", id))
}
