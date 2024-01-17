use poem::Result;
use poem_openapi::{param::Query, param::Header, payload::Json, payload::PlainText, OpenApi};
use liquid_breakout_backend::Backend;
use structs::{IdResponse, WhitelistInfo, WhitelistResponse};

mod structs;

pub struct Routes {
    backend: Backend
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

    // Map Test Whitelist
    #[oai(path = "/maptest/whitelist", method = "post")]
    pub async fn whitelist(&self, asset_id: Query<Option<u64>>) -> Result<WhitelistResponse> {
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

        let result = self.backend.whitelist_asset(asset_id, 0).await;
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
                    error: Some((*e).to_string()),
                    share_id: None   
                }
            )))
        }
    }

    // Map Test ID System
    #[oai(path = "/maptest/id/share", method = "get")]
    pub async fn get_shareable_id(&self, id: Query<Option<u64>>) -> Result<IdResponse> {
        let id = match id.0 {
            None => return Ok(IdResponse::InvalidId),
            Some(b) => b,
        };
        let result = self.backend.get_shareable_id(id.to_string());
        match result {
            Ok(share_id) => Ok(IdResponse::Ok(PlainText(share_id))),
            Err(e) => Ok(IdResponse::ServerError(PlainText((*e).to_string())))
        }
    }

    #[oai(path = "/maptest/id/number", method = "get")]
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
            Err(e) => Ok(IdResponse::ServerError(PlainText((*e).to_string())))
        }
    }

    // Map Test Map Hub System
    #[oai(path = "/maptest/hub/fetch", method = "get")]
    pub async fn fetch_maphub_data(&self) {

    }

    #[oai(path = "/maptest/hub/publish", method = "post")]
    pub async fn publish_to_maphub(&self) {

    }

    #[oai(path = "/maptest/hub/validatehash", method = "post")]
    pub async fn validate_hash_with_maphub(&self) {

    }
}