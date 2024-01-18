use poem_openapi::{payload::Json, payload::PlainText, ApiResponse, Object, Tags};

// Tags
#[derive(Tags)]
#[allow(dead_code)]
pub enum ApiTags {
    #[oai(rename = "Map Test Operation")]
    MapTestOperation,
    #[oai(rename = "Map Test - Map Hub")]
    MapTestMapHub,
    #[oai(rename = "Map Test - ID System")]
    MapTestIdSystem,
    #[oai(rename = "Ingame Moderation")]
    Moderation
}

// Map Test's Whitelist route
#[derive(Object)]
pub struct WhitelistInfo {
    pub success: bool,
    pub error: Option<String>,
    pub share_id: Option<String>
}

#[derive(ApiResponse)]
pub enum WhitelistResponse {
    #[oai(status = 200)]
    Ok(Json<WhitelistInfo>),

    #[oai(status = 400)]
    BadRequest(Json<WhitelistInfo>),

    #[oai(status = 500)]
    ServerError(Json<WhitelistInfo>)
}

// Map Test's ID system routes
#[derive(ApiResponse)]
pub enum IdResponse {
    #[oai(status = 200)]
    Ok(PlainText<String>),

    #[oai(status = 400)]
    InvalidId,

    #[oai(status = 401)]
    Unauthorized,

    #[oai(status = 500)]
    ServerError(PlainText<String>)
}