//! Infrastructure Adapter: `RuVectorMemoryAdapter` implements `MemorySink` against
//! RuVector's `VectorDB`.
//!
//! ADR-0112 (RuVector Adapter Isolation; extends ADR-0008); docs/ddd/08-memory-persistence.md.
//! Split out of the base `3db7fd2` monolith's combined `jitter-adapter-ruvnet`
//! package — here memory is isolated behind `MemorySink` only, with no witnessing
//! responsibility (that moved to `jitter-adapter-rvf`).

use jitter_ports::{MemorySink, PortError, ReceiptRef};
use ruvector_core::types::{DbOptions, HnswConfig};
use ruvector_core::{DistanceMetric, SearchQuery, VectorDB, VectorEntry};
use serde_json::Value;
use std::{collections::HashMap, path::Path};

const EMBEDDING_DIMENSIONS: usize = 4;

pub struct RuVectorMemoryAdapter {
    db: VectorDB,
}

impl RuVectorMemoryAdapter {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, PortError> {
        let requested = path.as_ref();
        let storage = if requested.is_dir() {
            requested.join("outcomes.redb")
        } else {
            requested.to_path_buf()
        };
        let options = DbOptions {
            dimensions: EMBEDDING_DIMENSIONS,
            distance_metric: DistanceMetric::Cosine,
            storage_path: storage.to_string_lossy().into_owned(),
            hnsw_config: Some(HnswConfig::default()),
            quantization: None,
        };
        VectorDB::new(options)
            .map(|db| Self { db })
            .map_err(|e| PortError::Unavailable(e.to_string()))
    }
}

impl MemorySink for RuVectorMemoryAdapter {
    fn record(&self, receipt: &ReceiptRef) -> Result<Vec<String>, PortError> {
        let vector = embedding(receipt);
        let query = SearchQuery {
            vector: vector.clone(),
            k: 3,
            filter: None,
            ef_search: Some(32),
        };
        let prior = self
            .db
            .search(query)
            .map_err(|e| PortError::Unavailable(e.to_string()))?
            .into_iter()
            .map(|entry| entry.id)
            .collect();

        let id = format!(
            "{}-{}",
            &receipt.scenario_digest[..12.min(receipt.scenario_digest.len())],
            &receipt.outcome_digest[..12.min(receipt.outcome_digest.len())]
        );
        let mut metadata = HashMap::<String, Value>::new();
        metadata.insert("authority".into(), Value::String(receipt.authority.clone()));
        metadata.insert(
            "scenario".into(),
            Value::String(receipt.scenario_digest.clone()),
        );
        let stored = self
            .db
            .insert(VectorEntry {
                id: Some(id),
                vector,
                metadata: Some(metadata),
            })
            .map_err(|e| PortError::Unavailable(e.to_string()))?;
        self.db
            .get(&stored)
            .map_err(|e| PortError::Unavailable(e.to_string()))?
            .ok_or_else(|| PortError::Unavailable("RuVector read-back missing".into()))?;
        Ok(prior)
    }
}

fn embedding(receipt: &ReceiptRef) -> Vec<f32> {
    vec![
        receipt.retry_amplification_factor as f32 / 10.0,
        receipt.p95_ticks as f32 / 1000.0,
        receipt.tenant_success_fairness.unwrap_or(-1.0) as f32,
        1.0,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn receipt() -> ReceiptRef {
        ReceiptRef {
            scenario_digest: "a".repeat(64),
            outcome_digest: "b".repeat(64),
            witness_digest: "c".repeat(64),
            authority: "none".into(),
            retry_amplification_factor: 1.5,
            p95_ticks: 20,
            tenant_success_fairness: Some(0.75),
        }
    }

    #[test]
    fn real_vector_round_trip_and_recall() {
        let dir = tempdir().unwrap();
        let memory = RuVectorMemoryAdapter::open(dir.path()).unwrap();
        assert!(memory.record(&receipt()).unwrap().is_empty());
        assert_eq!(memory.record(&receipt()).unwrap().len(), 1);
    }

    #[test]
    fn passes_shared_memory_sink_contract() {
        let dir = tempdir().unwrap();
        let memory = RuVectorMemoryAdapter::open(dir.path()).unwrap();
        jitter_contract_tests::assert_memory_sink(&memory);
    }
}
