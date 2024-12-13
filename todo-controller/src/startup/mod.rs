use crate::context::errors::AppError;
use crate::module::Modules;
use crate::routes::health_check::{hc, hc_postgres};
use crate::routes::todo::{
    create_todo, delete_todo, error_handler, find_todo, get_todo, update_todo, upsert_todo,
    TodoOpenApi,
};
use crate::routes::user::{create_user, get_user, get_user_by_username, UserOpenApi};
use axum::error_handling::HandleErrorLayer;
use axum::routing::get;
use axum::Router;
use dotenv::dotenv;
use std::env;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tower::{BoxError, ServiceBuilder};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use utoipa::openapi::{Info, OpenApiBuilder};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub async fn startup(modules: Arc<Modules>) {
    let mut openapi = OpenApiBuilder::default()
        .info(Info::new("axum-rusty API", "1.0.0"))
        .build();
    openapi.merge(TodoOpenApi::openapi());
    openapi.merge(UserOpenApi::openapi());

    let hc_router = Router::new()
        .route("/", get(hc))
        .route("/postgres", get(hc_postgres));

    let todo_router = Router::new()
        .route("/", get(find_todo).post(create_todo))
        .route(
            "/:id",
            get(get_todo)
                .patch(update_todo)
                .put(upsert_todo)
                .delete(delete_todo),
        );
    let user_router = Router::new()
        .route("/", get(get_user_by_username).post(create_user))
        .route("/:id", get(get_user));
    let cors = CorsLayer::new().allow_origin(Any);
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/swagger.json", openapi))
        .nest("/:v/hc", hc_router)
        .nest("/:v/todo", todo_router)
        .nest("/:v/user", user_router)
        .fallback(error_handler)
        .layer(cors)
        .layer(
            ServiceBuilder::new()
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
        )
        .with_state(modules);

    let addr = SocketAddr::from(init_addr());
    let listener = TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("TcpListener cannot bind."));
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .unwrap_or_else(|_| panic!("Server cannot launch."));
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
