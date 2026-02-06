use axum::{
    extract::{Multipart, Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::db::Database;
use crate::config::MAX_SKIN_SIZE;

pub type SharedDb = Arc<Database>;

#[derive(Deserialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub ok: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
}

pub fn api_routes(db: SharedDb) -> Router {
    Router::new()
        .route("/api/register", post(register))
        .route("/api/login", post(login))
        .route("/api/logout", post(logout))
        .route("/api/me", get(me))
        .route("/api/skin", post(upload_skin))
        .route("/api/skin/{id}", get(get_skin))
        .with_state(db)
}

async fn register(
    State(db): State<SharedDb>,
    Json(req): Json<AuthRequest>,
) -> impl IntoResponse {
    match db.register(&req.username, &req.password) {
        Ok(user) => Json(AuthResponse {
            ok: true,
            message: "Account created".into(),
            username: Some(user.username),
            user_id: Some(user.id),
        }),
        Err(e) => Json(AuthResponse {
            ok: false,
            message: e,
            username: None,
            user_id: None,
        }),
    }
}

async fn login(
    State(db): State<SharedDb>,
    jar: CookieJar,
    Json(req): Json<AuthRequest>,
) -> impl IntoResponse {
    match db.login(&req.username, &req.password) {
        Ok((user, token)) => {
            let cookie = Cookie::build(("session", token))
                .path("/")
                .http_only(false) // JS needs to read it for WS
                .max_age(time::Duration::days(7))
                .build();
            (
                jar.add(cookie),
                Json(AuthResponse {
                    ok: true,
                    message: "Logged in".into(),
                    username: Some(user.username),
                    user_id: Some(user.id),
                }),
            )
        }
        Err(e) => (
            jar,
            Json(AuthResponse {
                ok: false,
                message: e,
                username: None,
                user_id: None,
            }),
        ),
    }
}

async fn logout(
    State(db): State<SharedDb>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(cookie) = jar.get("session") {
        db.logout(cookie.value());
    }
    let removal = Cookie::build(("session", ""))
        .path("/")
        .max_age(time::Duration::seconds(0))
        .build();
    (
        jar.remove(removal),
        Json(AuthResponse {
            ok: true,
            message: "Logged out".into(),
            username: None,
            user_id: None,
        }),
    )
}

async fn me(
    State(db): State<SharedDb>,
    jar: CookieJar,
) -> impl IntoResponse {
    if let Some(cookie) = jar.get("session") {
        if let Some(user) = db.validate_session(cookie.value()) {
            return Json(AuthResponse {
                ok: true,
                message: "Authenticated".into(),
                username: Some(user.username),
                user_id: Some(user.id),
            });
        }
    }
    Json(AuthResponse {
        ok: false,
        message: "Not logged in".into(),
        username: None,
        user_id: None,
    })
}

async fn upload_skin(
    State(db): State<SharedDb>,
    jar: CookieJar,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let user = match jar.get("session").and_then(|c| db.validate_session(c.value())) {
        Some(u) => u,
        None => return (StatusCode::UNAUTHORIZED, "Not logged in").into_response(),
    };

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        if !content_type.starts_with("image/") {
            return (StatusCode::BAD_REQUEST, "Only image files allowed").into_response();
        }

        let data = match field.bytes().await {
            Ok(d) => d,
            Err(_) => return (StatusCode::BAD_REQUEST, "Failed to read file").into_response(),
        };

        if data.len() > MAX_SKIN_SIZE {
            return (StatusCode::BAD_REQUEST, "File too large (max 256KB)").into_response();
        }

        match db.set_skin(user.id, &data, &content_type) {
            Ok(_) => return (StatusCode::OK, "Skin uploaded").into_response(),
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response(),
        }
    }

    (StatusCode::BAD_REQUEST, "No file provided").into_response()
}

async fn get_skin(
    State(db): State<SharedDb>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match db.get_skin(id) {
        Some(skin) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, skin.mime),
             (header::CACHE_CONTROL, "public, max-age=300".into())],
            skin.data,
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "No skin found").into_response(),
    }
}
