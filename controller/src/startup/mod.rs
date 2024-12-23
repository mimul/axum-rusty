use crate::context::auth_resolver::auth;
use crate::context::errors::AppError;
use crate::module::Modules;
use crate::routes::health_check::{hc, hc_postgres};
use crate::routes::todo::{create_todo, delete_todo, find_todo, get_todo, update_todo, upsert_todo, TodoOpenApi};
use crate::routes::user::{create_user, get_user, get_user_by_username, login_user, UserOpenApi};
use axum::error_handling::HandleErrorLayer;
use axum::routing::{get, post};
use axum::{middleware, Json, Router};
use dotenv::dotenv;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use http::header::{ACCEPT, ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_REQUEST_HEADERS, ACCESS_CONTROL_REQUEST_METHOD, AUTHORIZATION, CONTENT_TYPE, ORIGIN};
use http::{HeaderValue, Method, StatusCode};
use serde_json::Value;
use tokio::net::TcpListener;
use tower::{BoxError, ServiceBuilder};
use tower_http::cors::{CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::openapi::{Info, OpenApiBuilder};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::context::api_response::ApiResponse;

pub async fn startup(modules: Arc<Modules>) {
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec![
            ORIGIN,
            AUTHORIZATION,
            ACCEPT,
            ACCESS_CONTROL_REQUEST_HEADERS,
            ACCESS_CONTROL_REQUEST_METHOD,
            CONTENT_TYPE,
            ACCESS_CONTROL_ALLOW_HEADERS,
        ])
        .expose_headers(vec![
            ORIGIN,
            AUTHORIZATION,
            ACCEPT,
            ACCESS_CONTROL_REQUEST_HEADERS,
            ACCESS_CONTROL_REQUEST_METHOD,
            CONTENT_TYPE,
            ACCESS_CONTROL_ALLOW_HEADERS,
        ])
        .allow_origin(
            modules
                .constants
                .allowed_origin
                .parse::<HeaderValue>()
                .unwrap(),
        );
    let mut openapi = OpenApiBuilder::default()
        .info(Info::new("axum-rusty API", "1.0.0"))
        .build();
    openapi.merge(TodoOpenApi::openapi());
    openapi.merge(UserOpenApi::openapi());

    let hc_router = Router::new()
        .route("/", get(hc))
        .route("/postgres", get(hc_postgres));

    let auth_router = Router::new()
        .route("/create", post(create_user))
        .route("/login", post(login_user));

    let todo_router = Router::new()
        .route("/", get(find_todo).post(create_todo),)
        .route("/:id", get(get_todo).patch(update_todo).put(upsert_todo).delete(delete_todo)
        ).route_layer(middleware::from_fn_with_state(modules.clone(), auth));

    let user_router = Router::new()
        .route("/", get(get_user_by_username)).route("/:id", get(get_user)
        ).route_layer(middleware::from_fn_with_state(modules.clone(), auth));

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/swagger.json", openapi))
        .nest("/:v/hc", hc_router)
        .nest("/:v/todo", todo_router)
        .nest("/:v/user", user_router)
        .nest("/:v/auth", auth_router)
        .fallback(fallback)
        .with_state(modules)
        .layer(cors)
        .layer(ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        AppError::Error("time out.".to_string())
                    } else {
                        AppError::Error(format!("Unhandled internal error: {error}"))
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        );

    let addr = SocketAddr::from(init_addr());
    let listener = TcpListener::bind(&addr).await
        .unwrap_or_else(|_| panic!("TcpListener cannot bind."));
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await
        .unwrap_or_else(|_| panic!("Server cannot launch."));
}

async fn fallback() -> Result<(StatusCode, Json<ApiResponse<Value>>), AppError> {
    Err(AppError::Error("abnormal uri".to_string()))
}

pub fn init_app() {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
}

fn init_addr() -> (IpAddr, u16) {
    let env_host = env::var_os("HOST").expect("HOST is undefined.");
    let ip_addr = env_host
        .into_string()
        .expect("HOST is invalid.")
        .parse::<IpAddr>()
        .expect("HOST is invalid.");

    let env_port = env::var_os("PORT").expect("PORT is undefined.");
    let port = env_port
        .into_string()
        .expect("PORT is invalid.")
        .parse::<u16>()
        .expect("PORT is invalid.");

    tracing::info!("Init ip address.");
    (ip_addr, port)
}
