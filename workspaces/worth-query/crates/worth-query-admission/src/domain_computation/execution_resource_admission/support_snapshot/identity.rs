//! Versioned support meaning; live capacity authority remains a separate pointer check.
use super::*;
pub(super) fn support_identity(
    provider: &WorthQueryExecutionProviderFamily,
    access_product: &WorthQueryExecutionAccessProductFamily,
    allocator: &WorthQueryExecutionAllocatorFamily,
    envelope: &WorthQueryExecutionResourceEnvelope,
    capacity: &dyn WorthQueryExecutionCapacityPort,
) -> Arc<str> {
    let mut parts = vec![
        "worth_query_execution_resource_support_v1".into(),
        format!("provider:{}", provider.as_str()),
        format!("access:{}", access_product.as_str()),
        format!("allocator:{}", allocator.as_str()),
        format!("capacity:{}", capacity.capacity_subject_identity()),
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
    ];
    if envelope.boundary()
        == worth_query_declaration::facade::domain_computation::WorthQueryExecutionBoundary::Atomic
    {
        parts[0] = "worth_query_execution_resource_support_v2".into();
        parts.push(format!("boundary:{}", envelope.boundary().as_str()));
        parts.push(format!(
            "scale-count:{}",
            envelope.scale_ceilings().iter().count()
        ));
        parts.push(format!(
            "resource-count:{}",
            envelope.resource_ceilings().iter().count()
        ));
    }
    Arc::<str>::from(hash_parts(&parts))
}
