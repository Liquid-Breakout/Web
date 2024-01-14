use poem_openapi::{param::Query, OpenApi};

use super::Routes;
use super::structs::BadResponse;

struct WhitelistResponse {}

#[OpenApi]
impl Routes {
    #[oai(path = "/whitelist", method = "post")]
    pub async fn whitelist(&self, asset_id: Query<Option<u64>>) -> Result<WhitelistResponse> {
        let asset_id = match asset_id.0 {
            None => return Ok(BadResponse),
            Some(b) => b,
        };

        self.backend.whitelist_asset(asset_id, 0);
    }
}