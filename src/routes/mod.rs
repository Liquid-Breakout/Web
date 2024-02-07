use poem::Result;
use poem_openapi::{param::Query, param::Header, payload::Json, payload::PlainText, OpenApi};
use liquid_breakout_backend::Backend;
use structs::{BanEntryObject, BanListResponse, IdResponse, WhitelistInfo, WhitelistResponse};

use self::structs::BanResponse;

mod structs;

pub struct Routes {
    backend: Backend
}

fn unbox_error(box_var: Box<dyn std::error::Error>) -> String {
    let unboxed = (*box_var).to_string();
    unboxed
}

#[OpenApi]
impl Routes {
    pub fn new(backend: Backend) -> Self {
        Self { backend: backend }
    }

    pub async fn authorized(&self, api_key: Header<Option<String>>) -> bool {
        let api_key = match api_key.0 {
            Some(k) => k,
            None => return false,
        };
        let valid = self.backend.is_valid_api_key(api_key.as_str()).await;
        match valid {
            Ok(valid) => {
                return valid
            },
            Err(_) => return false,
        };
    }

    // Moderation System
    #[oai(path = "/moderation/ban/list", method = "get", tag = structs::ApiTags::Moderation)]
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

    #[oai(path = "/moderation/ban", method = "post", tag = structs::ApiTags::Moderation)]
    pub async fn ban_player(&self, api_key: Header<Option<String>>, user_id: Query<Option<u64>>, duration: Query<Option<i32>>, moderator: Query<Option<String>>, reason: Query<Option<String>>) -> Result<BanResponse> {
        let authorized = self.authorized(api_key).await;
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

    #[oai(path = "/moderation/unban", method = "post", tag = structs::ApiTags::Moderation)]
    pub async fn unban_player(&self, api_key: Header<Option<String>>, user_id: Query<Option<u64>>) -> Result<BanResponse> {
        let authorized = self.authorized(api_key).await;
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
    #[oai(path = "/maptest/whitelist", method = "post", tag = structs::ApiTags::MapTestOperation)]
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
    #[oai(path = "/maptest/id/share", method = "get", tag = structs::ApiTags::MapTestIdSystem)]
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

    #[oai(path = "/maptest/id/number", method = "get", tag = structs::ApiTags::MapTestIdSystem)]
    pub async fn get_number_id(&self, api_key: Header<Option<String>>, id: Query<Option<String>>) -> Result<IdResponse> {
        let authorized = self.authorized(api_key).await;
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
    #[oai(path = "/maptest/hub/fetch", method = "get", tag = structs::ApiTags::MapTestMapHub)]
    pub async fn fetch_maphub_data(&self) {

    }

    #[oai(path = "/maptest/hub/publish", method = "post", tag = structs::ApiTags::MapTestMapHub)]
    pub async fn publish_to_maphub(&self) {

    }

    #[oai(path = "/maptest/hub/validatehash", method = "post", tag = structs::ApiTags::MapTestMapHub)]
    pub async fn validate_hash_with_maphub(&self) {

    }
}