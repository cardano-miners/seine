use chrono::{DateTime, TimeZone, Utc};
use miette::IntoDiagnostic;
use num_bigint::BigUint;
use num_traits::pow;
use serenity::all::{Colour, CreateEmbed};
use serenity::builder::ExecuteWebhook;
use serenity::http::Http;
use serenity::model::webhook::Webhook;

use crate::block::TunaBlock;

pub async fn send_webhook(
    url: &str,
    block: &TunaBlock,
    tx_hash: &str,
    block_hash: &str,
) -> miette::Result<()> {
    let http = Http::new("");

    let webhook = Webhook::from_url(&http, url).await.into_diagnostic()?;

    let description = format!(
        "Cardano Info\n- [Block](https://cexplorer.io/block/{})\n- [Transaction](https://cexplorer.io/tx/{})",
        block_hash,
        tx_hash
    );

    // Calculate the combined target
    let target = make_target(block.target_number, block.leading_zeros);

    let nonce = match block.nonce.as_deref() {
        Some(nonce) => format!("`{}`", nonce),
        None => "N/A".to_string(),
    };

    let miner_cred = block
        .payment_cred
        .as_deref()
        .or(block.nft_cred.as_deref())
        .map(|cred| format!("`{}`", cred))
        .unwrap_or_else(|| "N/A".to_string());

    let data = match block.data.as_deref() {
        Some(data) => format!("`{}`", data),
        None => "N/A".to_string(),
    };

    // Calculate the epoch
    let epoch = calculate_epoch(block.number);

    let formatted_time = Utc
        .timestamp_opt((block.current_posix_time / 1000) as i64, 0)
        .single() // This handles the Result
        .map(|dt: DateTime<Utc>| dt.format("%b %d, %Y, %H:%M:%S").to_string())
        .unwrap_or_else(|| "Invalid timestamp".to_string());

    let embed = CreateEmbed::new()
        .title(format!("New Block Mined: #{}", block.number))
        .description(description)
        .field("Hash", format!("`{}`", block.current_hash), false)
        .field("Target", format!("`{}`", target), false)
        .field("Nonce", nonce, false)
        .field("Miner Credential", miner_cred, false)
        .field("Data", data, false)
        .field("Epoch", epoch.to_string(), true)
        .field("Time", formatted_time, true)
        .colour(Colour::DARK_PURPLE);

    let builder = ExecuteWebhook::new().embed(embed);

    let _opt_message = webhook
        .execute(&http, false, builder)
        .await
        .into_diagnostic()?;

    Ok(())
}

fn make_target(target_number: u64, leading_zeros: u64) -> String {
    let base: BigUint = 16u64.into();
    let exponent = 60 - leading_zeros;
    let multiplier = pow(base, exponent as usize);
    let value = BigUint::from(target_number) * multiplier;
    format!("{:0>64x}", value)
}

fn calculate_epoch(block_number: u64) -> u64 {
    block_number / 2016 + 1
}
