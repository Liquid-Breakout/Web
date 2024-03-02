use full_moon::ast::{Call, Expression, FunctionArgs, Suffix};
use poem::Result;
use poem_openapi::{auth::ApiKey, param::Query, payload::Json, payload::PlainText, OpenApi, SecurityScheme};
use line_col::LineColLookup;
use liquid_breakout_backend::Backend;
use super::generic::{GenericRoutes, WebsocketIoQueue};
use super::structs::{ApiError, ApiTags, BanEntryObject, BanListResponse, BanRequestSchema, BanResponse, IdResponse, MaliciousScriptEntry, ScanMapInfo, ScanMapRequestSchema, ScanMapResponse, ScanMapResult, UnbanRequestSchema, WhitelistInfo, WhitelistRequestSchema, WhitelistResponse};

pub struct ApiRoutes {
    backend: Backend,
    generic_routes: GenericRoutes
}

fn unbox_error(box_var: Box<dyn std::error::Error>) -> String {
    let unboxed = (*box_var).to_string();
    unboxed
}

#[derive(SecurityScheme)]
#[oai(
    ty = "api_key",
    key_name = "x-api-key",
    key_in = "header"
)]
pub struct ApiKeyAuthorization(ApiKey);

#[OpenApi]
impl ApiRoutes {
    pub fn new(backend: Backend, generic_routes: &GenericRoutes) -> Self {
        Self { backend: backend, generic_routes: generic_routes.to_owned() }
    }

    pub async fn authorized(&self, api_key: ApiKey) -> bool {
        let valid = self.backend.is_valid_api_key(api_key.key.as_str()).await;
        match valid {
            Ok(valid) => {
                return valid
            },
            Err(_) => return false,
        };
    }

    // IO-related
    /*#[oai(path = "/websocket/io/send", method = "post")]
    pub async fn io_send(&self, action: Header<String>, bgm: Query<Option<String>>, ) {
        
    }*/

    // Moderation System
    #[oai(path = "/moderation/ban/list", method = "get", tag = ApiTags::Moderation)]
    pub async fn fetch_ban_list(&self) -> Result<BanListResponse> {
        let result = self.backend.get_ban_collection().await;
        match result {
            Ok(entries) => {
                // this is ugly, i wish there's a way to not do this
                let mut response_entries: Vec<BanEntryObject> = Vec::new();
                for entry in entries.into_iter() {
                    response_entries.push(BanEntryObject {
                        user_id: entry.user_id,
                        banned_time: entry.banned_time,
                        banned_until: entry.banned_until,
                        moderator: entry.moderator,
                        reason: entry.reason
                    })
                }

                Ok(BanListResponse::Ok(
                    Json(response_entries)
                ))
            },
            Err(e) => Ok(BanListResponse::ServerError(Json(ApiError { error: unbox_error(e) } )))
        }
    }

    #[oai(path = "/moderation/ban", method = "post", tag = ApiTags::Moderation)]
    pub async fn ban_player(&self, api_key: ApiKeyAuthorization, body: Json<BanRequestSchema>) -> Result<BanResponse> {
        let authorized = self.authorized(api_key.0).await;
        if !authorized {
            return Ok(BanResponse::Unauthorized)
        }

        let body = body.0;
        if body.user_id <= 0 {
            return Ok(BanResponse::BadRequest(Json(ApiError { error: "userId cannot be negative or 0.".to_string() })))
        }
        if body.duration < 0 && body.duration != -1 {
            return Ok(BanResponse::BadRequest(Json(ApiError { error: "duration can only be positive or -1.".to_string() })))
        }

        let result = self.backend.ban_player(body.user_id, body.duration, &body.moderator, &body.reason).await;
        match result {
            Ok(_) => Ok(BanResponse::Ok),
            Err(e) => Ok(BanResponse::ServerError(Json(ApiError { error: unbox_error(e) } )))
        }
    }

    #[oai(path = "/moderation/unban", method = "post", tag = ApiTags::Moderation)]
    pub async fn unban_player(&self, api_key: ApiKeyAuthorization, body: Json<UnbanRequestSchema>) -> Result<BanResponse> {
        let authorized = self.authorized(api_key.0).await;
        if !authorized {
            return Ok(BanResponse::Unauthorized)
        }

        let body = body.0;
        if body.user_id <= 0 {
            return Ok(BanResponse::BadRequest(Json(ApiError { error: "userId cannot be negative or 0.".to_string() })))
        }

        let result = self.backend.unban_player(body.user_id).await;
        match result {
            Ok(_) => Ok(BanResponse::Ok),
            Err(e) => Ok(BanResponse::ServerError(Json(ApiError { error: unbox_error(e) } )))
        }
    }

    // Map Test Scan Model
    #[oai(path = "/maptest/scanmap", method = "post", tag = ApiTags::MapTestOperation)]
    pub async fn scan_map(&self, api_key: ApiKeyAuthorization, body: Json<ScanMapRequestSchema>) -> Result<ScanMapResponse> {
        let authorized = self.authorized(api_key.0).await;
        if !authorized {
            return Ok(ScanMapResponse::Unauthorized)
        }

        let body = body.0;

        let bytes = match self.backend.download_asset_bytes(body.asset_id).await {
            Ok(bytes) => bytes,
            Err(e) => return Ok(ScanMapResponse::ServerError(Json(ApiError { error: unbox_error(e) } )))
        };

        let dom = match self.backend.dom_from_bytes(bytes) {
            Ok(dom) => dom,
            Err(e) => return Ok(ScanMapResponse::ServerError(Json(ApiError { error: unbox_error(e) } )))
        };

        let scripts = self.backend.dom_find_scripts(&dom);
        let mut result: Vec<MaliciousScriptEntry> = Vec::new();
        for (location, src) in scripts.into_iter() {
            let ast = match self.backend.luau_ast_from_string(&src) {
                Ok(ast) => ast,
                Err(e) => return Ok(ScanMapResponse::ServerError(Json(ApiError { error: unbox_error(e) } )))
            };
            let lookup = LineColLookup::new(&src);

            let found_getfenv = self.backend.luau_find_global_function_usage(&ast, "getfenv");
            if !found_getfenv.is_empty() {
                for ((pos, _), _) in found_getfenv.clone().into_iter() {
                    let (line, column) = lookup.get(pos);
                    result.push(MaliciousScriptEntry {
                        script: location.clone(),
                        line: line as u64,
                        column: column as u64,
                        reason: "Detected `getfenv` usage, which is extremely forbidden as it's commonly used for malicious purposes.".to_string(),
                    })
                }
            }

            let found_setfenv = self.backend.luau_find_global_function_usage(&ast, "setfenv");
            if !found_setfenv.is_empty() {
                for ((pos, _), _) in found_setfenv.clone().into_iter() {
                    let (line, column) = lookup.get(pos);
                    result.push(MaliciousScriptEntry {
                        script: location.clone(),
                        line: line as u64,
                        column: column as u64,
                        reason: "Detected `setfenv` usage, changing the script environment is not allowed.".to_string(),
                    })
                }
            }

            let found_require = self.backend.luau_find_global_function_usage(&ast, "require");
            if !found_require.is_empty() {
                for ((pos, _), suffixes) in found_require.clone().into_iter() {
                    let first_suffix = suffixes.first().unwrap();
                    if let Suffix::Call(call) = first_suffix {
                        if let Call::AnonymousCall(args) = call {
                            if let FunctionArgs::Parentheses { arguments, .. } = args {
                                if let Some(arg_pair) = arguments.first() {
                                    if let Expression::Number(token) = arg_pair.value() {
                                        if let Ok(id) = token.token().to_string().parse::<u64>() {
                                            let (line, column) = lookup.get(pos);
                                            result.push(MaliciousScriptEntry {
                                                script: location.clone(),
                                                line: line as u64,
                                                column: column as u64,
                                                reason: format!("Detected requiring by id ({}). This is used to download malicious scripts, thus is not allowed.", id),
                                            })
                                        }
                                    };
                                }
                            }
                        }
                    }
                }
            }
        };

        Ok(ScanMapResponse::Ok(Json(
            ScanMapInfo {
                result: ScanMapResult {
                    is_malicious: !result.is_empty(),
                    scripts: result
                }
            }
        )))
    }

    // Map Test Whitelist
    #[oai(path = "/maptest/whitelist", method = "post", tag = ApiTags::MapTestOperation)]
    pub async fn whitelist(&self, body: Json<WhitelistRequestSchema>) -> Result<WhitelistResponse> {
        let body = body.0;
        if body.user_id <= 0 {
            return Ok(WhitelistResponse::BadRequest(Json(ApiError { error: "userId cannot be negative or 0.".to_string() })))
        }

        let result = self.backend.whitelist_asset(body.asset_id, body.user_id).await;
        match result {
            Ok(_) => Ok(WhitelistResponse::Ok(Json(WhitelistInfo { share_id: self.backend.get_shareable_id(body.asset_id.to_string()).unwrap() }))),
            Err(e) => Ok(WhitelistResponse::ServerError(Json(ApiError { error: unbox_error(e) } )))
        }
    }

    // Map Test ID System
    #[oai(path = "/maptest/id/share", method = "get", tag = ApiTags::MapTestIdSystem)]
    pub async fn get_shareable_id(&self, id: Query<Option<u64>>) -> Result<IdResponse> {
        let id = match id.0 {
            None => return Ok(IdResponse::InvalidId(Json(ApiError { error: "Query `id` is not a number (u64).".to_string() } ))),
            Some(b) => b,
        };
        let result = self.backend.get_shareable_id(id.to_string());
        match result {
            Ok(share_id) => Ok(IdResponse::Ok(PlainText(share_id))),
            Err(e) => Ok(IdResponse::ServerError(Json(ApiError { error: unbox_error(e) } )))
        }
    }

    #[oai(path = "/maptest/id/number", method = "get", tag = ApiTags::MapTestIdSystem)]
    pub async fn get_number_id(&self, api_key: ApiKeyAuthorization, id: Query<Option<String>>) -> Result<IdResponse> {
        let authorized = self.authorized(api_key.0).await;
        if !authorized {
            return Ok(IdResponse::Unauthorized)
        }

        let id = match id.0 {
            None => return Ok(IdResponse::InvalidId(Json(ApiError { error: "Query `id` is not a String.".to_string() } ))),
            Some(b) => b,
        };
        let result = self.backend.get_number_id(id);
        match result {
            Ok(id) => Ok(IdResponse::Ok(PlainText(id.to_string()))),
            Err(e) => Ok(IdResponse::ServerError(Json(ApiError { error: unbox_error(e) } )))
        }
    }

    // Map Test Map Hub System
    #[oai(path = "/maptest/hub/fetch", method = "get", tag = ApiTags::MapTestMapHub)]
    pub async fn fetch_maphub_data(&self) {

    }

    #[oai(path = "/maptest/hub/publish", method = "post", tag = ApiTags::MapTestMapHub)]
    pub async fn publish_to_maphub(&self) {

    }

    #[oai(path = "/maptest/hub/validatehash", method = "post", tag = ApiTags::MapTestMapHub)]
    pub async fn validate_hash_with_maphub(&self) {

    }
}