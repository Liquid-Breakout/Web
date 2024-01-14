use poem::{endpoint::StaticFilesEndpoint, listener::TcpListener, Route};
use poem_openapi::OpenApiService;
use routes::Routes;

mod routes;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let api_service = OpenApiService::new(Routes, "Liquid Breakout API", "0.0.1")
        .server("https://api.liquidbreakout.com");
    let api_swagger = api_service.swagger_ui();

    let server_routes = Route::new()
        .nest("/", StaticFilesEndpoint::new("./assets"))
        .nest("/api", api_service)
        .nest("/api/docs", api_swagger);

    poem::Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(server_routes)
        .await
}