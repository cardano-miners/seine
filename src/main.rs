use std::env;

use miette::IntoDiagnostic;
use utxorpc::{spec::cardano::plutus_data::PlutusData, CardanoSyncClient, ClientBuilder, TipEvent};

use seine::{block::TunaBlock, database::Database, discord, extensions::*};

#[tokio::main]
async fn main() -> miette::Result<()> {
    let _ = dotenvy::dotenv().ok();

    let dolos_endpoint = env::var("DOLOS_ENDPOINT").into_diagnostic()?;
    let dolos_token = env::var("DOLOS_TOKEN").into_diagnostic()?;
    let account_id = env::var("CLOUDFLARE_ACCOUNT_ID").into_diagnostic()?;
    let database_id = env::var("CLOUDFLARE_DATABASE_ID").into_diagnostic()?;
    let d1_token = env::var("CLOUDFLARE_D1_TOKEN").into_diagnostic()?;
    let discord_webhook_url = env::var("DISCORD_WEBHOOK_URL").into_diagnostic()?;

    let db = Database::new(account_id, database_id, d1_token);

    loop {
        println!("connecting");

        let mut client = ClientBuilder::new()
            .uri(&dolos_endpoint)
            .into_diagnostic()?
            .metadata("dmtr-api-key", &dolos_token)
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
                                    let PlutusData::Constr(constr) = redeemer.plutus_data() else {
                                        miette::bail!("failed to decode tuna state");
                                    };

                                    let nonce =
                                        constr.fields[0].plutus_data.as_ref().and_then(|data| {
                                            match data {
                                                PlutusData::BoundedBytes(b) => Some(hex::encode(b)),
                                                _ => None,
                                            }
                                        });

                                    next_tuna_datum.nonce = nonce;
                                };

                                let block_hash = hex::encode(&header.hash);

                                let _resp = db
                                    .apply(&next_tuna_datum, &tx_hash, header.slot, &block_hash)
                                    .await;
                            }
                            TunaOutput::V2(tx_hash, output, inputs) => {
                                let mut next_tuna_datum: TunaBlock = output.datum().try_into()?;

                                let prev_block_info = inputs
                                    .iter()
                                    .filter(|input| input.is_tuna_v2())
                                    .find_map(|input| {
                                        input.redeemer.as_ref().map(|r| {
                                            // Update previous block info with nonce
                                            (next_tuna_datum.number - 1, r.clone())
                                        })
                                    });

                                if let Some((_block_number, redeemer)) = prev_block_info {
                                    let PlutusData::Constr(constr) = redeemer.plutus_data() else {
                                        miette::bail!("failed to decode tuna state");
                                    };

                                    let nonce =
                                        constr.fields[0].plutus_data.as_ref().and_then(|data| {
                                            match data {
                                                PlutusData::BoundedBytes(b) => Some(hex::encode(b)),
                                                _ => None,
                                            }
                                        });

                                    next_tuna_datum.nonce = nonce;

                                    let PlutusData::Constr(miner_cred) =
                                        constr.fields[1].plutus_data.as_ref().unwrap()
                                    else {
                                        todo!()
                                    };

                                    match miner_cred.tag {
                                        121 => {
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
                                        122 => {
                                            let PlutusData::BoundedBytes(policy) =
                                                miner_cred.fields[0].plutus_data.as_ref().unwrap()
                                            else {
                                                unreachable!()
                                            };

                                            let PlutusData::BoundedBytes(asset_name) =
                                                miner_cred.fields[1].plutus_data.as_ref().unwrap()
                                            else {
                                                unreachable!()
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
                                };

                                let block_hash = hex::encode(&header.hash);

                                db.apply(&next_tuna_datum, &tx_hash, header.slot, &block_hash)
                                    .await?;

                                discord::send_webhook(
                                    &discord_webhook_url,
                                    &next_tuna_datum,
                                    &tx_hash,
                                )
                                .await?;
                            }
                        }
                    }
                }
                TipEvent::Undo(block) => {
                    let (header, _body) = block.parts();

                    db.undo(header.slot).await?;
                }
                TipEvent::Reset(point) => {
                    db.reset(point).await?;
                }
            }
        }

        println!("disconnected");
    }
}
