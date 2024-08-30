use utxorpc::spec::cardano::{Asset, Block, BlockBody, BlockHeader, Multiasset, TxInput, TxOutput};

use crate::constants::{TUNA_V1_ADDRESS, TUNA_V1_POLICY_ID, TUNA_V2_ADDRESS, TUNA_V2_POLICY_ID};

pub trait BlockExtensions {
    fn parts(self) -> (BlockHeader, BlockBody);
}

impl BlockExtensions for Block {
    fn parts(self) -> (BlockHeader, BlockBody) {
        (self.header.unwrap(), self.body.unwrap())
    }
}

pub trait BlockBodyExtensions {
    fn outputs(self) -> impl Iterator<Item = TunaOutput>;
}

pub enum TunaOutput {
    V1(TxOutput, Vec<TxInput>),
    V2(TxOutput, Vec<TxInput>),
}

impl BlockBodyExtensions for BlockBody {
    fn outputs(self) -> impl Iterator<Item = TunaOutput> {
        self.tx
            .into_iter()
            .map(|tx| (tx.outputs, tx.inputs))
            .filter_map(|(outputs, inputs)| {
                for output in outputs {
                    if output.is_tuna_v2() {
                        return Some(TunaOutput::V2(output, inputs));
                    }

                    if output.is_tuna_v1() {
                        return Some(TunaOutput::V1(output, inputs));
                    }
                }

                None
            })
    }
}

pub trait TxOutputExtensions {
    fn is_tuna_v2(&self) -> bool;

    fn is_tuna_v1(&self) -> bool;
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
