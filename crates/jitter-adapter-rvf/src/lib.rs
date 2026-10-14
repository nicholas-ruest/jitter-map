//! Infrastructure Adapter: `RvfWitnessAdapter` implements `WitnessSink`, sealing a
//! run's outcome into a tamper-evident RVF witness chain.
//!
//! ADR-0113 (Witness Binding; extends ADR-0009); docs/ddd/07-evidence-receipts.md.
//! Split out of the base `3db7fd2` monolith's combined `jitter-adapter-ruvnet`
//! package, which folded RVF witnessing and RuVector memory into one crate — here
//! witnessing is isolated behind `WitnessSink` only.

use jitter_outcome::OutcomeSummary;
use jitter_ports::{PortError, WitnessRef, WitnessSink};
use rvf_crypto::{WitnessEntry, create_witness_chain, shake256_256, verify_witness_chain};

pub struct RvfWitnessAdapter;

impl WitnessSink for RvfWitnessAdapter {
    fn write(
        &self,
        outcome: &OutcomeSummary,
        scenario_digest: &str,
    ) -> Result<WitnessRef, PortError> {
        let actions: Vec<Vec<u8>> = vec![
            scenario_digest.as_bytes().to_vec(),
            format!("{outcome:?}").into_bytes(),
        ];
        let entries: Vec<WitnessEntry> = actions
            .iter()
            .enumerate()
            .map(|(i, action)| WitnessEntry {
                prev_hash: [0; 32],
                action_hash: shake256_256(action),
                timestamp_ns: i as u64,
                witness_type: if i == 0 { 1 } else { 2 },
            })
            .collect();
        let chain = create_witness_chain(&entries);
        verify_witness_chain(&chain).map_err(|e| PortError::Rejected(e.to_string()))?;
        Ok(WitnessRef {
            digest: hex::encode(shake256_256(&chain)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jitter_outcome::{AdmissionStrategy, OutcomeEvent};

    fn summary() -> OutcomeSummary {
        jitter_outcome::with_strategy(
            jitter_outcome::summarize(
                &[OutcomeEvent::Admitted {
                    tenant_id: 0,
                    request_id: 1,
                    tick: 1,
                }],
                1,
            ),
            AdmissionStrategy::AdaptiveBudget,
        )
    }

    #[test]
    fn real_witness_chain_seals_and_verifies() {
        let witness = RvfWitnessAdapter.write(&summary(), "digest").unwrap();
        assert_eq!(witness.digest.len(), 64);
    }

    #[test]
    fn digest_is_stable_across_repeated_seals() {
        let a = RvfWitnessAdapter.write(&summary(), "digest").unwrap();
        let b = RvfWitnessAdapter.write(&summary(), "digest").unwrap();
        assert_eq!(a.digest, b.digest);
    }

    #[test]
    fn passes_shared_witness_sink_contract() {
        jitter_contract_tests::assert_witness_sink(&RvfWitnessAdapter);
    }
}
