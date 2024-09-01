use miette::IntoDiagnostic;
use utxorpc::spec::sync::BlockRef;

use crate::{block::TunaBlock, constants::initial_point};

#[derive(Debug, serde::Deserialize)]
struct QueryResponse<T> {
    results: Vec<T>,
    success: bool,
}

#[derive(Debug, serde::Deserialize)]
struct TipPayload {
    cardano_hash: String,
    cardano_slot: u64,
}

#[derive(Debug, serde::Deserialize)]
struct TipResponse {
    result: Vec<QueryResponse<TipPayload>>,
    success: bool,
}

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
        let res: TipResponse = self
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

        if res.success && !res.result.is_empty() && res.result[0].success {
            let payload = &res.result[0].results[0];

            Ok(BlockRef {
                index: payload.cardano_slot,
                hash: hex::decode(&payload.cardano_hash).into_diagnostic()?.into(),
            })
        } else {
            Ok(initial_point())
        }
    }

    pub async fn insert_block(
        &self,
        block: &TunaBlock,
        cardano_tx_hash: &str,
        cardano_slot: u64,
        cardano_hash: &str,
    ) -> miette::Result<()> {
        let value: serde_json::Value = self
            .client
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
            .await
            .into_diagnostic()?
            .json()
            .await
            .into_diagnostic()?;

        if value["success"].as_bool().unwrap() {
            println!("inserted block {}", block.number);

            Ok(())
        } else {
            println!("failed to insert {}", block.number);

            miette::bail!("failed to insert block")
        }
    }
}
