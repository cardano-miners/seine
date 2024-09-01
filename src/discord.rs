use miette::IntoDiagnostic;
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

    let embed = CreateEmbed::new()
        .title(format!("New Block Mined: #{}", block.number))
        .description(format!(
            "Cardano Info\n- [Block](https://cexplorer.io/block/{})\n- [Transaction](https://cexplorer.io/tx/{})",
            block_hash,
            tx_hash
        ))
        .field("Hash", format!("`{}`", block.current_hash), false)
        .field("Leading Zeros", block.leading_zeros.to_string(), true)
        .field("Target Number", block.target_number.to_string(), true)
        .field("Epoch Time", block.epoch_time.to_string(), true)
        .field(
            "Current POSIX Time",
            block.current_posix_time.to_string(),
            true,
        )
        .field("Nonce", block.nonce.as_deref().unwrap_or("N/A"), true)
        .field(
            "Payment Credential",
            block.payment_cred.as_deref().unwrap_or("N/A"),
            true,
        )
        .field(
            "NFT Credential",
            block.nft_cred.as_deref().unwrap_or("N/A"),
            true,
        )
        .field("Data", block.data.as_deref().unwrap_or("N/A"), false)
        .colour(Colour::DARK_PURPLE);

    let builder = ExecuteWebhook::new().embed(embed);

    let _opt_message = webhook
        .execute(&http, false, builder)
        .await
        .into_diagnostic()?;

    Ok(())
}
