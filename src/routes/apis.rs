use poem::Result;
use poem_openapi::{auth::ApiKey, param::Query, param::Header, payload::Json, payload::PlainText, OpenApi, SecurityScheme};
use liquid_breakout_backend::Backend;
use super::generic::{GenericRoutes, WebsocketIoQueue};
use super::structs::{ApiTags, BanEntryObject, BanResponse, BanListResponse, IdResponse, WhitelistInfo, WhitelistResponse};

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
    key_name = "api_key",
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
    pub async fn io_send(&self, action: Header<String>, bgm: Query<Option<String>>, ) -> PlainText<&'static str> {
        PlainText("Welcome to Liquid Breakout Backend site. Visit /docs for documentation.")
    }*/

    // Moderation System
    #[oai(path = "/moderation/ban/list", method = "get", tag = ApiTags::Moderation)]
    pub async fn fetch_ban_list(&self) -> Result<BanListResponse> {
        let result = self.backend.get_ban_collection().await;
        match result {
            Ok(entries) => {
                // this is ugly, i wish there's a way to not do this
                let mut response_entries: Vec<BanEntryObject> = vec![];
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
            Err(e) => Ok(BanListResponse::ServerError(PlainText(unbox_error(e))))
        }
    }

    #[oai(path = "/moderation/ban", method = "post", tag = ApiTags::Moderation)]
    pub async fn ban_player(&self, api_key: ApiKeyAuthorization, user_id: Query<Option<u64>>, duration: Query<Option<i32>>, moderator: Query<Option<String>>, reason: Query<Option<String>>) -> Result<BanResponse> {
        let authorized = self.authorized(api_key.0).await;
        if !authorized {
            return Ok(BanResponse::Unauthorized)
        }

        let user_id = match user_id.0 {
            Some(i) => i,
            None => return Ok(BanResponse::InvalidUser(PlainText("Query `user_id` is not a number (u64).".to_string()))),
        };
        let duration_in_minutes = match duration.0 {
            Some(i) => i,
            None => return Ok(BanResponse::InvalidDuration(PlainText("Query `duration` is not a number (i32).".to_string()))),
        };
        if duration_in_minutes < 0 && duration_in_minutes != -1 {
            return Ok(BanResponse::InvalidDuration(PlainText("`duration` can only be positive or -1.".to_string())))
        }
        let moderator = match moderator.0 {
            Some(i) => i,
            None => return Ok(BanResponse::InvalidString(PlainText("Query `moderator` is not a String.".to_string()))),
        };
        let reason = match reason.0 {
            Some(i) => i,
            None => return Ok(BanResponse::InvalidString(PlainText("Query `reason` is not a String.".to_string()))),
        };

        let result = self.backend.ban_player(user_id, duration_in_minutes, &moderator, &reason).await;
        match result {
            Ok(_) => Ok(BanResponse::Ok),
            Err(e) => Ok(BanResponse::ServerError(PlainText(unbox_error(e))))
        }
    }

    #[oai(path = "/moderation/unban", method = "post", tag = ApiTags::Moderation)]
    pub async fn unban_player(&self, api_key: ApiKeyAuthorization, user_id: Query<Option<u64>>) -> Result<BanResponse> {
        let authorized = self.authorized(api_key.0).await;
        if !authorized {
            return Ok(BanResponse::Unauthorized)
        }

        let user_id = match user_id.0 {
            Some(i) => i,
            None => return Ok(BanResponse::InvalidUser(PlainText("Query `user_id` is not a number (u64).".to_string()))),
        };

        let result = self.backend.unban_player(user_id).await;
        match result {
            Ok(_) => Ok(BanResponse::Ok),
            Err(e) => Ok(BanResponse::ServerError(PlainText(unbox_error(e))))
        }
    }

    // Map Test Whitelist
    #[oai(path = "/maptest/whitelist", method = "post", tag = ApiTags::MapTestOperation)]
    pub async fn whitelist(&self, asset_id: Query<Option<u64>>, user_id: Query<Option<u64>>) -> Result<WhitelistResponse> {
        let asset_id = match asset_id.0 {
            None => return Ok(WhitelistResponse::BadRequest(
                Json(
                    WhitelistInfo {
                        success: false,
                        error: Some("Invalid Asset ID.".to_string()),
                        share_id: None   
                    }
                )
            )),
            Some(b) => b,
        };
        let user_id = match user_id.0 {
            None => return Ok(WhitelistResponse::BadRequest(
                Json(
                    WhitelistInfo {
                        success: false,
                        error: Some("Invalid User ID.".to_string()),
                        share_id: None   
                    }
                )
            )),
            Some(b) => b,
        };
        if user_id <= 0 {
            return Ok(WhitelistResponse::BadRequest(
                Json(
                    WhitelistInfo {
                        success: false,
                        error: Some("Invalid User ID.".to_string()),
                        share_id: None   
                    }
                )
            ))
        }

        let result = self.backend.whitelist_asset(asset_id, user_id).await;
        match result {
            Ok(_) => Ok(WhitelistResponse::Ok(
                Json(
                    WhitelistInfo {
                        success: true,
                        error: None,
                        share_id: Some(self.backend.get_shareable_id(asset_id.to_string()).unwrap())
                    }
                )
            )),
            Err(e) => Ok(WhitelistResponse::ServerError(Json(
                WhitelistInfo {
                    success: false,
                    error: Some(unbox_error(e)),
                    share_id: None   
                }
            )))
        }
    }

    // Map Test ID System
    #[oai(path = "/maptest/id/share", method = "get", tag = ApiTags::MapTestIdSystem)]
    pub async fn get_shareable_id(&self, id: Query<Option<u64>>) -> Result<IdResponse> {
        let id = match id.0 {
            None => return Ok(IdResponse::InvalidId),
            Some(b) => b,
        };
        let result = self.backend.get_shareable_id(id.to_string());
        match result {
            Ok(share_id) => Ok(IdResponse::Ok(PlainText(share_id))),
            Err(e) => Ok(IdResponse::ServerError(PlainText(unbox_error(e))))
        }
    }

    #[oai(path = "/maptest/id/number", method = "get", tag = ApiTags::MapTestIdSystem)]
    pub async fn get_number_id(&self, api_key: ApiKeyAuthorization, id: Query<Option<String>>) -> Result<IdResponse> {
        let authorized = self.authorized(api_key.0).await;
        if !authorized {
            return Ok(IdResponse::Unauthorized)
        }

        let id = match id.0 {
            Some(i) => i,
            None => return Ok(IdResponse::InvalidId),
        };
        let result = self.backend.get_number_id(id);
        match result {
            Ok(id) => Ok(IdResponse::Ok(PlainText(id.to_string()))),
            Err(e) => Ok(IdResponse::ServerError(PlainText(unbox_error(e))))
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