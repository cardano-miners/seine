use std::env;

use miette::IntoDiagnostic;
use utxorpc::{
    spec::cardano::{plutus_data::PlutusData, RedeemerPurpose},
    CardanoSyncClient, ClientBuilder, TipEvent,
};

use seine::{
    block::TunaBlock,
    constants::{TUNA_V1_POLICY_ID, TUNA_V2_POLICY_ID},
    database::Database,
    extensions::{
        BlockBodyExtensions, BlockExtensions, TunaOutput, TxInputExtensions, TxOutputExtensions,
    },
};

#[tokio::main]
async fn main() -> miette::Result<()> {
    let _ = dotenvy::dotenv().ok();

    let dolos_endpoint = env::var("DOLOS_ENDPOINT").into_diagnostic()?;
    let dolos_token = env::var("DOLOS_TOKEN").into_diagnostic()?;
    let account_id = env::var("CLOUDFLARE_ACCOUNT_ID").into_diagnostic()?;
    let database_id = env::var("CLOUDFLARE_DATABASE_ID").into_diagnostic()?;
    let d1_token = env::var("CLOUDFLARE_D1_TOKEN").into_diagnostic()?;

    let db = Database::new(account_id, database_id, d1_token);

    let mut client = ClientBuilder::new()
        .uri("http://localhost:50051")
        .into_diagnostic()?
        .metadata("dmtr-api-key", dolos_token)
        .into_diagnostic()?
        .build::<CardanoSyncClient>()
        .await;

    let intersect = db.tip().await?;

    let mut tip = client.follow_tip(vec![intersect]).await.into_diagnostic()?;

    while let Ok(event) = tip.event().await {
        match event {
            TipEvent::Apply(block) => {
                let (header, body) = block.parts();

                for tuna in body.outputs() {
                    match tuna {
                        TunaOutput::V1(tx_hash, output, inputs) => {
                            let mut next_tuna_datum: TunaBlock = output.datum().try_into()?;

                            let prev_block_info = inputs
                                .iter()
                                .filter(|input| input.is_tuna_v1())
                                .find_map(|input| {
                                    input.redeemer.as_ref().map(|r| {
                                        // Update previous block info with nonce
                                        (next_tuna_datum.number - 1, r.clone())
                                    })
                                });

                            if let Some((_block_number, redeemer)) = prev_block_info {
                                let PlutusData::Constr(constr) =
                                    redeemer.payload.unwrap().plutus_data.unwrap()
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

                            let _resp = db
                                .insert_block(
                                    next_tuna_datum,
                                    tx_hash,
                                    header.slot,
                                    hex::encode(&header.hash),
                                )
                                .await;
                        }
                        TunaOutput::V2(tx_hash, output, inputs) => {
                            let mut next_tuna_datum: TunaBlock = output.datum().try_into()?;

                            let prev_block_info = inputs.iter().find_map(|input| {
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
                                    redeemer.payload.unwrap().plutus_data.unwrap()
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

                                let _resp = db
                                    .insert_block(
                                        next_tuna_datum,
                                        tx_hash,
                                        header.slot,
                                        hex::encode(&header.hash),
                                    )
                                    .await;
                            };
                        }
                    }
                }
            }
            TipEvent::Undo(_block) => {}
            TipEvent::Reset(_point) => {}
        }
    }

    Ok(())
}
