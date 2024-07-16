use std::env;

use miette::IntoDiagnostic;
use utxorpc::{
    spec::{
        cardano::{plutus_data::PlutusData, RedeemerPurpose},
        sync::BlockRef,
    },
    CardanoSyncClient, ClientBuilder, TipEvent,
};

use seine::{
    block::TunaBlock,
    constants::{TUNA_V1_ADDRESS, TUNA_V1_POLICY_ID, TUNA_V2_ADDRESS, TUNA_V2_POLICY_ID},
    cursor,
};

#[tokio::main]
async fn main() -> miette::Result<()> {
    let _ = dotenvy::dotenv().ok();

    let dolos_endpoint = env::var("DOLOS_ENDPOINT").into_diagnostic()?;
    let account_id = env::var("CLOUDFLARE_ACCOUNT_ID").into_diagnostic()?;
    let database_id = env::var("CLOUDFLARE_DATABASE_ID").into_diagnostic()?;
    let d1_token = env::var("CLOUDFLARE_D1_TOKEN").into_diagnostic()?;
    let d1_endpoint = format!(
        "https://api.cloudflare.com/client/v4/accounts/{account_id}/d1/database/{database_id}/query"
    );

    let mut client = ClientBuilder::new()
        .uri(dolos_endpoint)
        .into_diagnostic()?
        .build::<CardanoSyncClient>()
        .await;

    let cursor = cursor::Cursor::new();

    let intersect = cursor.get().await;

    let post_client = reqwest::Client::new();

    let mut tip = client.follow_tip(vec![intersect]).await.into_diagnostic()?;

    while let Ok(event) = tip.event().await {
        match event {
            TipEvent::Apply(block) => {
                let body = block.body.unwrap();
                let header = block.header.unwrap();

                let intersect = BlockRef {
                    hash: header.hash,
                    index: header.slot,
                };

                for tx in body.tx {
                    for output in tx.outputs {
                        if output.address == TUNA_V1_ADDRESS {
                            let contains_tuna_state = output.assets.iter().any(|multi_asset| {
                                multi_asset.policy_id == TUNA_V1_POLICY_ID
                                    && multi_asset
                                        .assets
                                        .iter()
                                        .any(|asset| asset.name == "lord tuna".as_bytes())
                            });

                            if !contains_tuna_state {
                                continue;
                            }

                            let mut next_tuna_datum: TunaBlock =
                                output.datum.unwrap().plutus_data.unwrap().try_into()?;

                            let prev_block_info = tx.inputs.iter().find_map(|input| {
                                let contains_tuna_state =
                                    input.as_output.iter().next().unwrap().assets.iter().any(
                                        |multi_asset| {
                                            multi_asset.policy_id == TUNA_V1_POLICY_ID
                                                && multi_asset.assets.iter().any(|asset| {
                                                    asset.name == "lord tuna".as_bytes()
                                                })
                                        },
                                    );

                                if !contains_tuna_state {
                                    return None;
                                }

                                input.redeemer.as_ref().and_then(|r| {
                                    if r.purpose() == RedeemerPurpose::Spend {
                                        // Update previous block info with nonce
                                        Some((next_tuna_datum.number - 1, r.clone()))
                                    } else {
                                        None
                                    }
                                })
                            });

                            if let Some((_block_number, redeemer)) = prev_block_info {
                                let PlutusData::Constr(constr) =
                                    redeemer.datum.unwrap().plutus_data.unwrap()
                                else {
                                    miette::bail!("failed to decode tuna state");
                                };

                                let nonce = constr.fields[0].plutus_data.as_ref().and_then(
                                    |data| match data {
                                        PlutusData::BoundedBytes(b) => Some(hex::encode(b)),
                                        _ => None,
                                    },
                                );

                                next_tuna_datum.nonce = nonce;
                            };

                            println!("{:#?}", next_tuna_datum);

                            let _resp = post_client.post(&d1_endpoint).bearer_auth(&d1_token).json(
                                &serde_json::json!({
                                    "sql": "INSERT INTO blocks (number, hash, leading_zeros, target_number, epoch_time, current_posix_time, nonce) VALUES (?, ?, ?, ?, ?, ?, ?)",
                                    "params": [
                                        next_tuna_datum.number,
                                        next_tuna_datum.current_hash,
                                        next_tuna_datum.leading_zeros,
                                        next_tuna_datum.target_number,
                                        next_tuna_datum.epoch_time,
                                        next_tuna_datum.current_posix_time,
                                        next_tuna_datum.nonce,
                                    ]
                                }),
                            ).send().await;
                        } else if output.address == TUNA_V2_ADDRESS {
                            let contains_tuna_state = output.assets.iter().any(|multi_asset| {
                                multi_asset.policy_id == TUNA_V2_POLICY_ID
                                    && multi_asset
                                        .assets
                                        .iter()
                                        .any(|asset| asset.name.slice(0..4) == "TUNA".as_bytes())
                            });

                            if !contains_tuna_state {
                                continue;
                            }

                            let mut next_tuna_datum: TunaBlock =
                                output.datum.unwrap().plutus_data.unwrap().try_into()?;

                            let prev_block_info = tx.inputs.iter().find_map(|input| {
                                let contains_tuna_state =
                                    input.as_output.iter().next().unwrap().assets.iter().any(
                                        |multi_asset| {
                                            multi_asset.policy_id == TUNA_V2_POLICY_ID
                                                && multi_asset.assets.iter().any(|asset| {
                                                    asset.name.slice(0..4) == "TUNA".as_bytes()
                                                })
                                        },
                                    );

                                if !contains_tuna_state {
                                    return None;
                                }

                                input.redeemer.as_ref().and_then(|r| {
                                    if r.purpose() == RedeemerPurpose::Spend {
                                        // Update previous block info with nonce
                                        Some((next_tuna_datum.number - 1, r.clone()))
                                    } else {
                                        None
                                    }
                                })
                            });

                            if let Some((_block_number, redeemer)) = prev_block_info {
                                let PlutusData::Constr(constr) =
                                    redeemer.datum.unwrap().plutus_data.unwrap()
                                else {
                                    miette::bail!("failed to decode tuna state");
                                };

                                let nonce = constr.fields[0].plutus_data.as_ref().and_then(
                                    |data| match data {
                                        PlutusData::BoundedBytes(b) => Some(hex::encode(b)),
                                        _ => None,
                                    },
                                );

                                let PlutusData::Constr(miner_cred) =
                                    constr.fields[1].plutus_data.as_ref().unwrap()
                                else {
                                    todo!()
                                };

                                match miner_cred.tag {
                                    0 => {
                                        let PlutusData::BoundedBytes(b) =
                                            miner_cred.fields[0].plutus_data.as_ref().unwrap()
                                        else {
                                            todo!()
                                        };

                                        let _data = miner_cred.fields[1]
                                            .plutus_data
                                            .as_ref()
                                            .unwrap()
                                            .clone();

                                        next_tuna_datum.payment_cred = Some(hex::encode(b));
                                        // next_tuna_datum.data =
                                        // Some(serde_json::json!(data).to_string());
                                    }
                                    1 => {
                                        let PlutusData::BoundedBytes(policy) =
                                            miner_cred.fields[0].plutus_data.as_ref().unwrap()
                                        else {
                                            todo!()
                                        };

                                        let PlutusData::BoundedBytes(asset_name) =
                                            miner_cred.fields[1].plutus_data.as_ref().unwrap()
                                        else {
                                            todo!()
                                        };

                                        let _data = miner_cred.fields[3]
                                            .plutus_data
                                            .as_ref()
                                            .unwrap()
                                            .clone();

                                        next_tuna_datum.nft_cred = Some(format!(
                                            "{}{}",
                                            hex::encode(policy),
                                            hex::encode(asset_name)
                                        ));
                                    }
                                    _ => unreachable!(),
                                }

                                next_tuna_datum.nonce = nonce;

                                println!("{:#?}", next_tuna_datum);

                                let _resp = post_client.post(&d1_endpoint).bearer_auth(&d1_token).json(
                                    &serde_json::json!({
                                        "sql": "INSERT INTO blocks (number, hash, leading_zeros, target_number, epoch_time, current_posix_time, nonce, miner_cred, nft_cred, data) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                        "params": [
                                            next_tuna_datum.number,
                                            next_tuna_datum.current_hash,
                                            next_tuna_datum.leading_zeros,
                                            next_tuna_datum.target_number,
                                            next_tuna_datum.epoch_time,
                                            next_tuna_datum.current_posix_time,
                                            next_tuna_datum.nonce,
                                            next_tuna_datum.payment_cred,
                                            next_tuna_datum.nft_cred,
                                            next_tuna_datum.data,
                                        ]
                                    }),
                                ).send().await;
                            };
                        }
                    }
                }

                cursor.set(intersect).await;
            }
            TipEvent::Undo(_block) => {}
            TipEvent::Reset(_point) => {}
        }
    }

    Ok(())
}
