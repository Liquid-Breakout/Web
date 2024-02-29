use poem_openapi::{payload::Json, payload::PlainText, ApiResponse, Object, Tags};

fn default_user_id() -> i64 {
    1
}

fn default_user_id_u64() -> u64 {
    1
}

fn default_moderator() -> String {
    "cutymeo".to_string()
}
fn default_reason() -> String {
    "Misbehave moment".to_string()
}

// Request schemas
#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct BanRequestSchema {
    #[oai(default = "default_user_id_u64", rename = "userId")]
    pub user_id: u64,
    pub duration: i32,
    #[oai(default = "default_moderator")]
    pub moderator: String,
    #[oai(default = "default_reason")]
    pub reason: String,
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct UnbanRequestSchema {
    #[oai(default = "default_user_id_u64", rename = "userId")]
    pub user_id: u64
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct ScanMapRequestSchema {
    #[oai(rename = "assetId")]
    pub asset_id: u64
}

#[derive(Debug, Object, Clone, Eq, PartialEq)]
pub struct WhitelistRequestSchema {
    #[oai(rename = "assetId")]
    pub asset_id: u64,
    #[oai(default = "default_user_id_u64", rename = "userId")]
    pub user_id: u64
}

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

// General
fn default_error() -> String {
    "your request is sussy".to_string()
}

#[derive(Object)]
pub struct ApiError {
    #[oai(default = "default_error")]
    pub error: String
}

// API-specifics

// Moderation's Ban List
#[derive(Object)]
pub struct BanEntryObject {
    #[oai(default = "default_user_id", rename = "userId")]
    pub user_id: i64,
    #[oai(rename = "bannedTime")]
    pub banned_time: i64,
    #[oai(rename = "bannedUntil")]
    pub banned_until: i64,
    #[oai(default = "default_moderator")]
    pub moderator: String,
    #[oai(default = "default_reason")]
    pub reason: String,
}

#[derive(ApiResponse)]
pub enum BanListResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<BanEntryObject>>),

    #[oai(status = 500)]
    ServerError(Json<ApiError>)
}

// Moderation's Ban and Unban
#[derive(ApiResponse)]
pub enum BanResponse {
    #[oai(status = 200)]
    Ok,

    #[oai(status = 400)]
    BadRequest(Json<ApiError>),

    #[oai(status = 401)]
    Unauthorized,

    #[oai(status = 500)]
    ServerError(Json<ApiError>)
}

// Map Test's Whitelist
fn default_share_id() -> String {
    "abcdef".to_string()
}

#[derive(Object)]
pub struct WhitelistInfo {
    #[oai(default = "default_share_id", rename = "shareableId")]
    pub share_id: String
}

#[derive(ApiResponse)]
pub enum WhitelistResponse {
    #[oai(status = 200)]
    Ok(Json<WhitelistInfo>),

    #[oai(status = 400)]
    BadRequest(Json<ApiError>),

    #[oai(status = 500)]
    ServerError(Json<ApiError>)
}

// Map Test's Scan Map
fn default_script() -> String {
    "Model.Abc.Def".to_string()
}
fn default_line_col() -> u64 {
    1
}
fn default_malicious_reason() -> String {
    "Used forbidden function".to_string()
}

#[derive(Object)]
pub struct MaliciousScriptEntry {
    #[oai(default = "default_script")]
    pub script: String,
    #[oai(default = "default_line_col")]
    pub line: u64,
    #[oai(default = "default_line_col")]
    pub column: u64,
    #[oai(default = "default_malicious_reason")]
    pub reason: String
}

fn default_malicious_result() -> bool {
    false
}

#[derive(Object)]
pub struct ScanMapResult {
    #[oai(default = "default_malicious_result", rename = "isMalicious")]
    pub is_malicious: bool,
    pub scripts: Vec<MaliciousScriptEntry>
}

#[derive(Object)]
pub struct ScanMapInfo {
    pub result: ScanMapResult
}

#[derive(ApiResponse)]
pub enum ScanMapResponse {
    #[oai(status = 200)]
    Ok(Json<ScanMapInfo>),

    #[oai(status = 400)]
    InvalidId(Json<ApiError>),

    #[oai(status = 401)]
    Unauthorized,

    #[oai(status = 500)]
    ServerError(Json<ApiError>)
}

// Map Test's ID system routes
#[derive(ApiResponse)]
pub enum IdResponse {
    #[oai(status = 200)]
    Ok(PlainText<String>),

    #[oai(status = 400)]
    InvalidId(Json<ApiError>),

    #[oai(status = 401)]
    Unauthorized,

    #[oai(status = 500)]
    ServerError(Json<ApiError>)
}