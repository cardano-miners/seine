use miette::IntoDiagnostic;
use utxorpc::spec::sync::BlockRef;

use crate::{block::TunaBlock, constants::initial_point};

pub struct Database {
    client: reqwest::Client,
    endpoint: String,
    d1_token: String,
}

impl Database {
    pub fn new(account_id: String, database_id: String, d1_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            endpoint: format!(
                "https://api.cloudflare.com/client/v4/accounts/{account_id}/d1/database/{database_id}/query"
            ),
            d1_token,
        }
    }

    pub async fn tip(&self) -> miette::Result<BlockRef> {
        let res: serde_json::Value = self
            .client
            .post(&self.endpoint)
            .bearer_auth(&self.d1_token)
            .json(&serde_json::json!({
                "sql": r#"
                    SELECT cardano_slot, cardano_hash
                    FROM blocks
                    ORDER BY number DESC
                    LIMIT 1
                "#,
            }))
            .send()
            .await
            .into_diagnostic()?
            .json()
            .await
            .into_diagnostic()?;

        dbg!(res);

        Ok(initial_point())
    }

    pub async fn insert_block(
        &self,
        block: TunaBlock,
        cardano_tx_hash: String,
        cardano_slot: u64,
        cardano_hash: String,
    ) -> Result<serde_json::Value, reqwest::Error> {
        self.client
            .post(&self.endpoint)
            .bearer_auth(&self.d1_token)
            .json(&serde_json::json!({
                "sql": r#"
                    INSERT INTO blocks (
                        number, hash, leading_zeros,
                        target_number, epoch_time,
                        current_posix_time, nonce, miner_cred,
                        nft_cred, data, cardano_tx_hash, cardano_slot,
                        cardano_hash
                      )
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                "params": [
                    block.number,
                    block.current_hash,
                    block.leading_zeros,
                    block.target_number,
                    block.epoch_time,
                    block.current_posix_time,
                    block.nonce,
                    block.payment_cred,
                    block.nft_cred,
                    block.data,
                    cardano_tx_hash,
                    cardano_slot,
                    cardano_hash,
                ]
            }))
            .send()
            .await?
            .json()
            .await
    }
}
