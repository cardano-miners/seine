use serde::{Deserialize, Serialize};
use utxorpc::spec::cardano::{big_int::BigInt, plutus_data::PlutusData};

#[derive(Debug, Serialize, Deserialize)]
pub struct TunaBlock {
    pub number: u64,
    pub current_hash: String,
    pub leading_zeros: u64,
    pub target_number: u64,
    pub epoch_time: u64,
    pub current_posix_time: u64,
    pub nonce: Option<String>,
    pub payment_cred: Option<String>,
    pub nft_cred: Option<String>,
    pub data: Option<String>,
}

impl TunaBlock {
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    pub fn to_json_pretty(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

impl TryFrom<PlutusData> for TunaBlock {
    type Error = miette::Error;

    fn try_from(value: PlutusData) -> Result<Self, Self::Error> {
        let PlutusData::Constr(constr) = value else {
            miette::bail!("failed to decode tuna state");
        };

        let Some(number) = constr.fields[0]
            .plutus_data
            .as_ref()
            .and_then(|data| match data {
                PlutusData::BigInt(w) => w.big_int.as_ref(),
                _ => None,
            })
            .and_then(|i| match i {
                BigInt::Int(n) => Some(*n as u64),
                _ => None,
            })
        else {
            miette::bail!("failed to decode tuna state.number");
        };

        let Some(current_hash) =
            constr.fields[1]
                .plutus_data
                .as_ref()
                .and_then(|data| match data {
                    PlutusData::BoundedBytes(b) => Some(hex::encode(b)),
                    _ => None,
                })
        else {
            miette::bail!("failed to decode tuna state.number");
        };

        let Some(leading_zeros) = constr.fields[2]
            .plutus_data
            .as_ref()
            .and_then(|data| match data {
                PlutusData::BigInt(w) => w.big_int.as_ref(),
                _ => None,
            })
            .and_then(|i| match i {
                BigInt::Int(n) => Some(*n as u64),
                _ => None,
            })
        else {
            miette::bail!("failed to decode tuna state.number");
        };

        let Some(target_number) = constr.fields[3]
            .plutus_data
            .as_ref()
            .and_then(|data| match data {
                PlutusData::BigInt(w) => w.big_int.as_ref(),
                _ => None,
            })
            .and_then(|i| match i {
                BigInt::Int(n) => Some(*n as u64),
                _ => None,
            })
        else {
            miette::bail!("failed to decode tuna state.number");
        };

        let Some(epoch_time) = constr.fields[4]
            .plutus_data
            .as_ref()
            .and_then(|data| match data {
                PlutusData::BigInt(w) => w.big_int.as_ref(),
                _ => None,
            })
            .and_then(|i| match i {
                BigInt::Int(n) => Some(*n as u64),
                _ => None,
            })
        else {
            miette::bail!("failed to decode tuna state.number");
        };

        let Some(current_posix_time) = constr.fields[5]
            .plutus_data
            .as_ref()
            .and_then(|data| match data {
                PlutusData::BigInt(w) => w.big_int.as_ref(),
                _ => None,
            })
            .and_then(|i| match i {
                BigInt::Int(n) => Some(*n as u64),
                _ => None,
            })
        else {
            miette::bail!("failed to decode tuna state.number");
        };

        Ok(TunaBlock {
            number,
            current_hash,
            leading_zeros,
            target_number,
            epoch_time,
            current_posix_time,
            nonce: None,
            payment_cred: None,
            nft_cred: None,
            data: None,
        })
    }
}
