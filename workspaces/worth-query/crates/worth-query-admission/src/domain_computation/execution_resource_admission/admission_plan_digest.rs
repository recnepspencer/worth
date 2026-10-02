use worth_query_installation::facade::WorthQueryExecutionResourceEnvelope;

use crate::admission_digest::hash_parts;

pub(super) fn admitted_plan_identity(
    binding_identity: &str,
    contract_identity: &str,
    request_identity: &str,
    support_identity: &str,
    strategy_name: &str,
    envelope_identity: &str,
) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for (prefix, value) in [
        ("", "worth_query_admitted_execution_resource_plan_v1"),
        ("binding:", binding_identity),
        ("contract:", contract_identity),
        ("request:", request_identity),
        ("support:", support_identity),
        ("strategy:", strategy_name),
        ("envelope:", envelope_identity),
    ] {
        hasher.update(((prefix.len() + value.len()) as u64).to_le_bytes());
        hasher.update(prefix.as_bytes());
        hasher.update(value.as_bytes());
    }
    worth_foundational::facade::CanonicalDigestId::new(hasher.finalize().into()).render_hex()
}

pub(super) fn admitted_envelope_identity(envelope: &WorthQueryExecutionResourceEnvelope) -> String {
    hash_parts(&[
        "worth_query_admitted_execution_resource_envelope_v1".into(),
        format!("mode:{}", envelope.mode().as_str()),
        format!("safe-point:{}", envelope.cancellation_safe_point().as_str()),
        format!(
            "degradation:{}",
            envelope
                .degradation()
                .map_or("complete", |degradation| degradation.as_str())
        ),
        format!(
            "partial-effect:{}",
            envelope.partial_effect_posture().as_str()
        ),
        format!(
            "yielded-state:{}",
            envelope.yielded_state_posture().as_str()
        ),
        format!(
            "retained-progress:{}",
            envelope.retained_progress_posture().as_str()
        ),
        format!(
            "scale:{}",
            envelope
                .scale_ceilings()
                .iter()
                .map(|(axis, value)| format!("{}={value}", axis.as_str()))
                .collect::<Vec<_>>()
                .join(",")
        ),
        format!(
            "resources:{}",
            envelope
                .resource_ceilings()
                .iter()
                .map(|(dimension, value)| format!("{}={value}", dimension.as_str()))
                .collect::<Vec<_>>()
                .join(",")
        ),
    ])
}
