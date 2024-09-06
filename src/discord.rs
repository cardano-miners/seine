use miette::IntoDiagnostic;
use serenity::all::{Colour, CreateEmbed};
use serenity::builder::ExecuteWebhook;
use serenity::http::Http;
use serenity::model::webhook::Webhook;

use crate::block::TunaBlock;

pub async fn send_webhook(url: &str, block: &TunaBlock, tx_hash: &str) -> miette::Result<()> {
    let http = Http::new("");

    let webhook = Webhook::from_url(&http, url).await.into_diagnostic()?;

    // Calculate the epoch
    let epoch = calculate_epoch(block.number);

    let embed = CreateEmbed::new()
        .title(format!(
            "[New Block Mined: #{}](https://cexplorer.io/tx/{})",
            block.number, tx_hash
        ))
        .field("Epoch", epoch.to_string(), true)
        .colour(Colour::DARK_PURPLE);

    let builder = ExecuteWebhook::new().embed(embed);

    let _opt_message = webhook
        .execute(&http, false, builder)
        .await
        .into_diagnostic()?;

    Ok(())
}

fn calculate_epoch(block_number: u64) -> u64 {
    // We passed 15 epochs before hard forking.
    if block_number <= 15 * 2016 {
        block_number / 2016 + 1
    } else {
        (block_number - (15 * 2016)) / 504 + 16
    }
}
