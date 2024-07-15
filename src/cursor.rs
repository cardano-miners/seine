use std::{env, path::PathBuf};

use utxorpc::spec::sync::BlockRef;

use crate::constants::initial_point;

pub struct Cursor {
    path: PathBuf,
}

impl Cursor {
    pub fn new() -> Self {
        let path = env::var("SEINE_CURSOR")
            .unwrap_or_else(|_| "./cursor".to_string())
            .into();

        Self { path }
    }

    pub async fn get(&self) -> BlockRef {
        tokio::fs::read(&self.path)
            .await
            .ok()
            .and_then(|data| serde_json::from_slice(&data).ok())
            .unwrap_or_else(initial_point)
    }

    pub async fn set(&self, cursor: BlockRef) {
        tokio::fs::write(&self.path, serde_json::to_vec(&cursor).unwrap())
            .await
            .unwrap();
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}
