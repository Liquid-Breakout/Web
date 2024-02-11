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

#[derive(Object)]
pub struct BanEntryObject {
    pub user_id: i64,
    pub banned_time: i64,
    pub banned_until: i64,
    pub moderator: String,
    pub reason: String,
}

// Moderation's Ban List
#[derive(ApiResponse)]
pub enum BanListResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<BanEntryObject>>),

    #[oai(status = 500)]
    ServerError(PlainText<String>)
}

// Moderation's Ban and Unban
#[derive(ApiResponse)]
pub enum BanResponse {
    #[oai(status = 200)]
    Ok,

    #[oai(status = 400)]
    InvalidUser(PlainText<String>),
    #[oai(status = 400)]
    InvalidDuration(PlainText<String>),
    #[oai(status = 400)]
    InvalidString(PlainText<String>),

    #[oai(status = 401)]
    Unauthorized,

    #[oai(status = 500)]
    ServerError(PlainText<String>)
}

// Map Test's Whitelist
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

// Map Test's Scan Map
#[derive(Object)]
pub struct MaliciousScriptEntry {
    pub script: String,
    pub line: u64,
    pub column: u64,
    pub reason: String
}

#[derive(Object)]
pub struct ScanMapResult {
    pub is_malicious: bool,
    pub scripts: Vec<MaliciousScriptEntry>
}

#[derive(Object)]
pub struct ScanMapInfo {
    pub success: bool,
    pub result: Option<ScanMapResult>,
    pub error: Option<String>
}

#[derive(ApiResponse)]
pub enum ScanMapResponse {
    #[oai(status = 200)]
    Ok(Json<ScanMapInfo>),

    #[oai(status = 400)]
    InvalidId(Json<ScanMapInfo>),

    #[oai(status = 401)]
    Unauthorized,

    #[oai(status = 500)]
    ServerError(Json<ScanMapInfo>)
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