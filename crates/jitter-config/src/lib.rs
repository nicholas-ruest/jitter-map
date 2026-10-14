//! Cross-Cutting Support: typed, validated configuration for composition roots.
//!
//! ADR-0117 (Typed Config Boundary); docs/ddd/11-security-policy.md. Fails fast on
//! any missing/invalid required field; no silent defaults for required fields
//! (docs/implementation-contract.md). Used only by `jitter-cli` and `jitter-service`
//! — never named by `jitter-application` or any domain crate.

use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("required environment variable {0} is not set")]
    Missing(&'static str),
    #[error("environment variable {field} has an invalid value: {reason}")]
    Invalid { field: &'static str, reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppConfig {
    /// Required: where the RuVector-backed outcome memory lives. No silent default —
    /// an operator must decide this, because it determines what auditable history a
    /// run joins.
    pub memory_path: PathBuf,
    pub aws_max_attempts: u32,
    pub timeout_ms: u64,
    pub flywheel_feed_path: PathBuf,
}

const MEMORY_PATH: &str = "JITTER_MEMORY_PATH";
const AWS_MAX_ATTEMPTS: &str = "JITTER_AWS_MAX_ATTEMPTS";
const TIMEOUT_MS: &str = "JITTER_TIMEOUT_MS";
const FLYWHEEL_FEED_PATH: &str = "JITTER_FLYWHEEL_FEED_PATH";

/// Loads configuration from the process environment.
pub fn load() -> Result<AppConfig, ConfigError> {
    let env: HashMap<String, String> = std::env::vars().collect();
    load_from(&env)
}

/// Loads configuration from an explicit environment map — the form every test in
/// this crate uses, so tests never mutate real process environment state.
pub fn load_from(env: &HashMap<String, String>) -> Result<AppConfig, ConfigError> {
    let memory_path = env
        .get(MEMORY_PATH)
        .ok_or(ConfigError::Missing(MEMORY_PATH))?
        .into();

    let aws_max_attempts = match env.get(AWS_MAX_ATTEMPTS) {
        None => 3,
        Some(raw) => raw.parse().map_err(|_| ConfigError::Invalid {
            field: AWS_MAX_ATTEMPTS,
            reason: format!("{raw:?} is not a positive integer"),
        })?,
    };
    if aws_max_attempts == 0 {
        return Err(ConfigError::Invalid {
            field: AWS_MAX_ATTEMPTS,
            reason: "must be at least 1".into(),
        });
    }

    let timeout_ms = match env.get(TIMEOUT_MS) {
        None => 5_000,
        Some(raw) => raw.parse().map_err(|_| ConfigError::Invalid {
            field: TIMEOUT_MS,
            reason: format!("{raw:?} is not a non-negative integer"),
        })?,
    };

    let flywheel_feed_path = env
        .get(FLYWHEEL_FEED_PATH)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("evidence/flywheel-feed.jsonl"));

    Ok(AppConfig {
        memory_path,
        aws_max_attempts,
        timeout_ms,
        flywheel_feed_path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_with(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn missing_required_field_fails_fast() {
        let env = env_with(&[]);
        assert_eq!(
            load_from(&env).unwrap_err(),
            ConfigError::Missing(MEMORY_PATH)
        );
    }

    #[test]
    fn minimal_valid_config_uses_documented_defaults() {
        let env = env_with(&[(MEMORY_PATH, "/tmp/mem")]);
        let config = load_from(&env).unwrap();
        assert_eq!(config.memory_path, PathBuf::from("/tmp/mem"));
        assert_eq!(config.aws_max_attempts, 3);
        assert_eq!(config.timeout_ms, 5_000);
    }

    #[test]
    fn invalid_aws_max_attempts_fails_before_any_adapter_is_constructed() {
        let env = env_with(&[
            (MEMORY_PATH, "/tmp/mem"),
            (AWS_MAX_ATTEMPTS, "not-a-number"),
        ]);
        assert!(matches!(
            load_from(&env),
            Err(ConfigError::Invalid {
                field: AWS_MAX_ATTEMPTS,
                ..
            })
        ));
    }

    #[test]
    fn zero_aws_max_attempts_is_invalid() {
        let env = env_with(&[(MEMORY_PATH, "/tmp/mem"), (AWS_MAX_ATTEMPTS, "0")]);
        assert!(matches!(
            load_from(&env),
            Err(ConfigError::Invalid {
                field: AWS_MAX_ATTEMPTS,
                ..
            })
        ));
    }

    #[test]
    fn invalid_timeout_fails_fast() {
        let env = env_with(&[(MEMORY_PATH, "/tmp/mem"), (TIMEOUT_MS, "-1")]);
        assert!(matches!(
            load_from(&env),
            Err(ConfigError::Invalid {
                field: TIMEOUT_MS,
                ..
            })
        ));
    }

    #[test]
    fn explicit_overrides_are_honored() {
        let env = env_with(&[
            (MEMORY_PATH, "/tmp/mem"),
            (AWS_MAX_ATTEMPTS, "7"),
            (TIMEOUT_MS, "9000"),
            (FLYWHEEL_FEED_PATH, "/tmp/feed.jsonl"),
        ]);
        let config = load_from(&env).unwrap();
        assert_eq!(config.aws_max_attempts, 7);
        assert_eq!(config.timeout_ms, 9000);
        assert_eq!(config.flywheel_feed_path, PathBuf::from("/tmp/feed.jsonl"));
    }
}
