use jitter_ports::PortError;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FederationEnvelope {
    untrusted: bool,
    pub(crate) provenance: String,
    pub(crate) relay: String,
    pub(crate) retrieved_at: String,
    pub(crate) data: FederationData,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FederationData {
    pub(crate) count: usize,
    messages: Vec<Value>,
}

pub(crate) fn parse_federation(
    output: &[u8],
) -> Result<(FederationEnvelope, BTreeMap<String, u64>), PortError> {
    let envelope: FederationEnvelope = serde_json::from_slice(output)
        .map_err(|error| PortError::Rejected(format!("invalid Ruflo federation JSON: {error}")))?;
    if !envelope.untrusted {
        return Err(PortError::Rejected(
            "Ruflo federation response lost its untrusted label".into(),
        ));
    }
    if !envelope.relay.starts_with("wss://") {
        return Err(PortError::Rejected(
            "Ruflo federation returned a non-TLS relay".into(),
        ));
    }
    if envelope.provenance.trim().is_empty() || envelope.retrieved_at.trim().is_empty() {
        return Err(PortError::Rejected(
            "Ruflo federation provenance is incomplete".into(),
        ));
    }
    if envelope.data.count != envelope.data.messages.len() {
        return Err(PortError::Rejected(
            "Ruflo federation count does not match messages".into(),
        ));
    }
    let mut message_types = BTreeMap::new();
    for message in &envelope.data.messages {
        let object = message
            .as_object()
            .ok_or_else(|| PortError::Rejected("Ruflo message is not an object".into()))?;
        let kind = if let Some(kind) = object
            .get("type")
            .and_then(Value::as_str)
            .filter(|kind| !kind.trim().is_empty())
        {
            kind
        } else if object
            .get("raw")
            .and_then(Value::as_str)
            .is_some_and(|raw| !raw.trim().is_empty())
        {
            "opaque-raw"
        } else {
            return Err(PortError::Rejected("Ruflo message type is invalid".into()));
        };
        *message_types.entry(kind.to_owned()).or_insert(0) += 1;
    }
    Ok((envelope, message_types))
}
