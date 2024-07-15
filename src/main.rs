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
    constants::{initial_point, TUNA_V1_ADDRESS, TUNA_V1_POLICY_ID},
};

#[tokio::main]
async fn main() -> miette::Result<()> {
    let _ = dotenvy::dotenv().ok();

    let dolos_endpoint = env::var("DOLOS_ENDPOINT").into_diagnostic()?;

    let mut client = ClientBuilder::new()
        .uri(dolos_endpoint)
        .into_diagnostic()?
        .build::<CardanoSyncClient>()
        .await;

    let intersect = get_cursor().await;

    let mut tip = client.follow_tip(vec![intersect]).await.into_diagnostic()?;

    while let Ok(event) = tip.event().await {
        match event {
            TipEvent::Apply(block) => {
                let body = block.body.unwrap();

                for tx in body.tx {
                    for output in tx.outputs {
                        if output.address != TUNA_V1_ADDRESS {
                            continue;
                        }

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

                        let mut tuna_datum: TunaBlock =
                            output.datum.unwrap().plutus_data.unwrap().try_into()?;

                        let redeemer = tx.inputs.iter().find_map(|input| {
                            input.redeemer.as_ref().and_then(|r| {
                                if r.purpose() == RedeemerPurpose::Spend {
                                    input.redeemer.as_ref().cloned()
                                } else {
                                    None
                                }
                            })
                        });

                        if let Some(redeemer) = redeemer {
                            let PlutusData::Constr(constr) =
                                redeemer.datum.unwrap().plutus_data.unwrap()
                            else {
                                miette::bail!("failed to decode tuna state");
                            };

                            let Some(nonce) =
                                constr.fields[0]
                                    .plutus_data
                                    .as_ref()
                                    .and_then(|data| match data {
                                        PlutusData::BoundedBytes(b) => Some(hex::encode(b)),
                                        _ => None,
                                    })
                            else {
                                miette::bail!("failed to decode tuna state.number");
                            };

                            tuna_datum.nonce = Some(nonce)
                        };

                        println!("{:#?}", tuna_datum);
                    }
                }
            }
            TipEvent::Undo(_block) => {}
            TipEvent::Reset(_point) => {}
        }
    }

    Ok(())
}

async fn get_cursor() -> BlockRef {
    tokio::fs::read("./cursor")
        .await
        .ok()
        .and_then(|data| serde_json::from_slice(&data).ok())
        .unwrap_or_else(initial_point)
}
