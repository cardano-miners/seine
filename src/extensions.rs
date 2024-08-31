use utxorpc::{
    spec::cardano::{
        plutus_data::PlutusData, Asset, Block, BlockBody, BlockHeader, Multiasset, TxInput,
        TxOutput,
    },
    ChainBlock,
};

use crate::constants::{TUNA_V1_ADDRESS, TUNA_V1_POLICY_ID, TUNA_V2_ADDRESS, TUNA_V2_POLICY_ID};

pub trait BlockExtensions {
    fn parts(self) -> (BlockHeader, BlockBody);
}

impl BlockExtensions for ChainBlock<Block> {
    fn parts(self) -> (BlockHeader, BlockBody) {
        let block = self.parsed.unwrap();

        (block.header.unwrap(), block.body.unwrap())
    }
}

pub trait BlockBodyExtensions {
    fn outputs(self) -> impl Iterator<Item = TunaOutput>;
}

pub enum TunaOutput {
    V1(String, TxOutput, Vec<TxInput>),
    V2(String, TxOutput, Vec<TxInput>),
}

impl BlockBodyExtensions for BlockBody {
    fn outputs(self) -> impl Iterator<Item = TunaOutput> {
        self.tx
            .into_iter()
            .map(|tx| (hex::encode(tx.hash), tx.outputs, tx.inputs))
            .filter_map(|(tx_hash, outputs, inputs)| {
                for output in outputs {
                    if output.is_tuna_v2() {
                        return Some(TunaOutput::V2(tx_hash, output, inputs));
                    }

                    if output.is_tuna_v1() {
                        return Some(TunaOutput::V1(tx_hash, output, inputs));
                    }
                }

                None
            })
    }
}

pub trait TxInputExtensions {
    fn is_tuna_v1(&self) -> bool;
}

impl TxInputExtensions for TxInput {
    fn is_tuna_v1(&self) -> bool {
        dbg!(self)
            .as_output
            .as_ref()
            .map(|output| output.is_tuna_v1())
            .unwrap_or(false)
    }
}

pub trait TxOutputExtensions {
    fn is_tuna_v2(&self) -> bool;

    fn is_tuna_v1(&self) -> bool;

    fn datum(self) -> PlutusData;
}

impl TxOutputExtensions for TxOutput {
    fn is_tuna_v2(&self) -> bool {
        self.address == TUNA_V2_ADDRESS
            && is_lord_tuna(&self.assets, TUNA_V2_POLICY_ID, |asset| {
                asset.name.slice(0..4) == "TUNA".as_bytes()
            })
    }

    fn is_tuna_v1(&self) -> bool {
        self.address == TUNA_V1_ADDRESS
            && is_lord_tuna(&self.assets, TUNA_V1_POLICY_ID, |asset| {
                asset.name == "lord tuna".as_bytes()
            })
    }

    fn datum(self) -> PlutusData {
        self.datum.unwrap().payload.unwrap().plutus_data.unwrap()
    }
}

fn is_lord_tuna<'a, A, F>(assets: A, policy_id: &[u8], filter: F) -> bool
where
    A: IntoIterator<Item = &'a Multiasset>,
    F: FnMut(&Asset) -> bool + Copy,
{
    assets.into_iter().any(|multi_asset| {
        multi_asset.policy_id == policy_id && multi_asset.assets.iter().any(filter)
    })
}
