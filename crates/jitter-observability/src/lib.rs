//! Cross-Cutting Support: structured tracing instrumentation for composition roots.
//!
//! ADR-0118 (Cross-Cutting Observability); docs/ddd/12-operations-failure.md. Never
//! panics when tracing is disabled (docs/implementation-contract.md).
//!
//! **Documented deviation**: `record_tick_metrics` is shown in
//! docs/implementation-contract.md taking `&TickResult` (a `jitter-scheduler` type).
//! This crate takes the tick number and `&[OutcomeEvent]` directly instead, so a
//! generic cross-cutting crate depends only on `jitter-outcome`, not on the
//! scheduler's internal result type.

use jitter_config::AppConfig;
use jitter_outcome::OutcomeEvent;
use tracing_subscriber::EnvFilter;

/// Held for the lifetime of the process; dropping it is harmless (there is nothing
/// to flush beyond what `tracing_subscriber`'s global dispatcher already owns).
pub struct Guard;

/// Installs a global tracing subscriber, writing to stderr. `config` is accepted
/// (rather than ignored) so the span of "what observability depends on" stays
/// visible at the call site, even though today only the environment's `RUST_LOG`
/// filter is read.
///
/// Writing to stderr (not stdout) is load-bearing for `jitter-cli`: ADR-0024 froze
/// stdout as the CLI's machine-readable JSON contract, and both this crate's own
/// tracing events *and* any `tracing`-instrumented dependency (RuVector/`hnsw_rs`
/// emit their own spans) would otherwise interleave with that JSON on the same
/// stream.
pub fn init_tracing(_config: &AppConfig) -> Guard {
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .try_init();
    Guard
}

/// Emits one tracing event per outcome produced in `tick`, each carrying
/// `tenant_id`/`request_id`/`admitted` fields.
pub fn record_tick_metrics(tick: u32, events: &[OutcomeEvent]) {
    let span = tracing::info_span!("tick", tick);
    let _enter = span.enter();
    for event in events {
        let (tenant_id, request_id, admitted, outcome) = match *event {
            OutcomeEvent::Admitted {
                tenant_id,
                request_id,
                ..
            } => (tenant_id, request_id, true, "admitted"),
            OutcomeEvent::Retried {
                tenant_id,
                request_id,
                ..
            } => (tenant_id, request_id, false, "retried"),
            OutcomeEvent::Deferred {
                tenant_id,
                request_id,
                ..
            } => (tenant_id, request_id, false, "deferred"),
            OutcomeEvent::Denied {
                tenant_id,
                request_id,
                ..
            } => (tenant_id, request_id, false, "denied"),
            OutcomeEvent::Exhausted {
                tenant_id,
                request_id,
                ..
            } => (tenant_id, request_id, false, "exhausted"),
            OutcomeEvent::HorizonExceeded {
                tenant_id,
                request_id,
                ..
            } => (tenant_id, request_id, false, "horizon_exceeded"),
        };
        tracing::info!(tenant_id, request_id, admitted, outcome, "tick outcome");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone, Default)]
    struct BufferWriter(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for BufferWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for BufferWriter {
        type Writer = Self;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    #[test]
    fn tick_outcomes_emit_a_span_with_expected_fields() {
        let buffer = BufferWriter::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(buffer.clone())
            .with_ansi(false)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            record_tick_metrics(
                1,
                &[
                    OutcomeEvent::Admitted {
                        tenant_id: 3,
                        request_id: 9,
                        tick: 1,
                    },
                    OutcomeEvent::Denied {
                        tenant_id: 4,
                        request_id: 10,
                        tick: 1,
                    },
                ],
            );
        });
        let output = String::from_utf8(buffer.0.lock().unwrap().clone()).unwrap();
        assert!(output.contains("tenant_id=3"), "{output}");
        assert!(output.contains("admitted=true"), "{output}");
        assert!(output.contains("admitted=false"), "{output}");
    }

    #[test]
    fn init_tracing_does_not_panic_when_called_repeatedly() {
        let config = jitter_config::load_from(&std::collections::HashMap::from([(
            "JITTER_MEMORY_PATH".to_string(),
            "/tmp/mem".to_string(),
        )]))
        .unwrap();
        let _guard = init_tracing(&config);
        let _guard_again = init_tracing(&config);
    }

    #[test]
    fn empty_event_list_emits_no_outcome_lines_without_panicking() {
        record_tick_metrics(0, &[]);
    }
}
