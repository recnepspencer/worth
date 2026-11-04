//! Exact, versioned node payload for the installed inbound contract selector.

use std::num::NonZeroU64;

use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::facade::application_program::ApplicationWorkflowInboundRef;
use worth_query_declaration::facade::application_schema::{
    ApplicationInboundOccurrenceLimits, ApplicationInboundOccurrenceProtocol,
};

pub(super) fn encode(inbound: &ApplicationWorkflowInboundRef) -> String {
    let limits = inbound.limits();
    serde_json::json!([
        2,
        inbound.protocol().identity().as_str(),
        inbound.protocol().version().get(),
        inbound.source_identity(),
        limits.maximum_envelope_bytes.get(),
        limits.maximum_verifier_work.get(),
        limits.maximum_payload_bytes.get(),
        limits.maximum_outstanding_dispatch_provenance.get(),
        limits.maximum_accepted_occurrences.get(),
        limits.maximum_accepted_bytes.get(),
        limits.maximum_concurrent_publications.get(),
        limits.maximum_discovery_work.get(),
        limits.replay_window_milliseconds.get(),
        limits.maximum_cleanup_work.get(),
    ])
    .to_string()
}

pub(super) fn decode(
    encoded: &str,
) -> Option<(
    ApplicationInboundOccurrenceProtocol,
    String,
    ApplicationInboundOccurrenceLimits,
)> {
    let values: Vec<serde_json::Value> = serde_json::from_str(encoded).ok()?;
    if values.len() != 14 || values[0].as_u64()? != 2 {
        return None;
    }
    let identity = BoundaryProtocolIdentity::parse(values[1].as_str()?.to_owned()).ok()?;
    let version =
        BoundaryProtocolVersion::try_new(u32::try_from(values[2].as_u64()?).ok()?).ok()?;
    let source = values[3].as_str()?.to_owned();
    if source.trim().is_empty() {
        return None;
    }
    let number = |index: usize| NonZeroU64::new(values[index].as_u64()?);
    let limits = ApplicationInboundOccurrenceLimits {
        maximum_envelope_bytes: number(4)?,
        maximum_verifier_work: number(5)?,
        maximum_payload_bytes: number(6)?,
        maximum_outstanding_dispatch_provenance: number(7)?,
        maximum_accepted_occurrences: number(8)?,
        maximum_accepted_bytes: number(9)?,
        maximum_concurrent_publications: number(10)?,
        maximum_discovery_work: number(11)?,
        replay_window_milliseconds: number(12)?,
        maximum_cleanup_work: number(13)?,
    };
    if !limits.accommodates_payload() {
        return None;
    }
    Some((
        ApplicationInboundOccurrenceProtocol::new(identity, version),
        source,
        limits,
    ))
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn retained_contract_rejects_corrupt_versions_limits_and_extra_fields() {
        let valid = serde_json::json!([
            2,
            "worth.query.workflow.remote",
            1,
            "rail",
            1024,
            1024,
            256,
            8,
            8,
            2048,
            2,
            16,
            1000,
            16,
        ]);
        assert!(decode(&valid.to_string()).is_some());
        for (index, replacement) in [
            (0, serde_json::json!(1)),
            (2, serde_json::json!(0)),
            (3, serde_json::json!("")),
            (6, serde_json::json!(4096)),
            (7, serde_json::json!(0)),
        ] {
            let mut changed = valid.clone();
            changed[index] = replacement;
            assert!(
                decode(&changed.to_string()).is_none(),
                "field {index} was accepted"
            );
        }
        let mut extra = valid.as_array().unwrap().clone();
        extra.push(serde_json::json!(7));
        assert!(decode(&serde_json::Value::Array(extra).to_string()).is_none());
        let mut old = valid.as_array().unwrap().clone();
        old[0] = serde_json::json!(1);
        old.remove(5);
        assert!(decode(&serde_json::Value::Array(old).to_string()).is_none());
    }
}
