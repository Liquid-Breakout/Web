use std::env;
use poem::{endpoint::StaticFilesEndpoint, listener::TcpListener, Route};
use poem_openapi::OpenApiService;
use routes::Routes;

use liquid_breakout_backend::Backend;

mod routes;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Check for environment variables
    let roblox_cookie = env::var("ROBLOX_COOKIE").expect("Server cannot start: Failed to read ROBLOX_COOKIE from environment");
    let mongodb_url = env::var("MONGODB_URL").expect("Server cannot start: Failed to read MONGODB_URL from environment");

    println!("Server starting up.");

    let mut backend = Backend::new(
        roblox_cookie,
        vec![
            "123456789*=+-aAbBcCdDeEfFgGhHiIjJkKlLmMnNoOpPqQrRsStTuUvVwWxXyYzZ".to_string(),
            "0123456789".to_string()
        ]
    );
    let connect_result = backend.connect_mongodb(mongodb_url).await;
    match connect_result {
        Ok(_) => {},
        Err(e) => panic!("Server cannot start: Failed to connect to MongoDB, reason: {}", (*e).to_string())
    }

    let routes = Routes::new(backend);

    let api_service = OpenApiService::new(routes, "Liquid Breakout API", "0.0.1")
        .server("https://api.liquidbreakout.com");
    let api_swagger = api_service.swagger_ui();

    let server_routes = Route::new()
        .nest("/", StaticFilesEndpoint::new("./assets").index_file("index.html"))
        .nest("/api", api_service)
        .nest("/api/docs", api_swagger);

    println!("Starting Server, listening at port: 3000");

    poem::Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(server_routes)
        .await
}