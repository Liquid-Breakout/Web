use poem_openapi::OpenApi;

pub mod structs;

pub struct Routes;
#[OpenApi]
impl Routes {}