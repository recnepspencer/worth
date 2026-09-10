use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::{WorthQueryImpactSemanticProjection, WorthQueryInvalidationSemanticAccessKey};

pub(super) fn impact_identity(
    class: crate::domain_installation::WorthQueryImpactClass,
    roles: &[crate::domain_installation::WorthQuerySemanticDependencyRole],
    affected_dependency_count: usize,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::ConsumerInvalidationDelta)
        .field_shape(WorthQueryEvidenceTag::new("projection"), "impact")
        .field_value(
            WorthQueryEvidenceTag::new("impact-class"),
            class.canonical_name(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("impact-roles"),
            roles.iter().map(|role| role.canonical_name()),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("affected-dependencies"),
            affected_dependency_count,
        )
        .seal()
}

pub(super) fn invalidation_identity(
    delta: &super::super::WorthQueryConsumerInvalidationDelta,
    impact: &WorthQueryImpactSemanticProjection,
    keys: &[WorthQueryInvalidationSemanticAccessKey],
) -> WorthQueryEvidenceIdentity {
    let conditional_path = delta.conditional_provenance();
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::ConsumerInvalidationDelta)
        .field_shape(WorthQueryEvidenceTag::new("projection"), "semantic-delta")
        .field_evidence_identity(WorthQueryEvidenceTag::new("impact"), impact.identity())
        .field_value(
            WorthQueryEvidenceTag::new("disposition"),
            delta.disposition().canonical_name(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("cause"),
            delta.cause().canonical_name(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("locality"),
            delta.locality().canonical_name(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("continuation"),
            delta.continuation().canonical_name(),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("delivery-causes"),
            delta
                .cause()
                .delivery_causes()
                .iter()
                .map(|cause| cause.as_str()),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("key-contract"),
            keys.iter().map(|key| key.contract_key().as_str()),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("key-contract-identity"),
            keys.iter()
                .map(|key| key.contract_identity().0.to_string()),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("key-contract-revision"),
            keys.iter()
                .map(|key| key.contract_revision().0.to_string()),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("key-field-path"),
            keys.iter()
                .map(|key| key.field_path().terminal_projection_for_boundary()),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("key-shape"),
            keys.iter().map(|key| aspect_shape_name(key.expected_shape())),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("key-absence"),
            keys.iter().map(|key| absence_name(key.absence())),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("key-lane"),
            keys.iter().map(|key| native_lane_name(key.lane())),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("conditional-path"),
            conditional_path.iter().map(
                crate::domain_installation::operation_execution::workflow_conditional_trace::conditional_trace_semantic_material,
            ),
        )
        .seal()
}

fn aspect_shape_name(value: worth_foundational::facade::AspectValuePosture) -> String {
    use worth_foundational::facade::AspectValuePosture;
    match value {
        AspectValuePosture::Scalar(kind) => format!("scalar:{}", kind.canonical_name()),
        AspectValuePosture::Struct => "struct".to_owned(),
        AspectValuePosture::Absent(law) => format!("absent:{}", absence_name(law)),
    }
}

fn absence_name(value: worth_foundational::facade::AbsenceLaw) -> &'static str {
    match value {
        worth_foundational::facade::AbsenceLaw::Required => "required",
        worth_foundational::facade::AbsenceLaw::Optional => "optional",
        worth_foundational::facade::AbsenceLaw::Defaulted => "defaulted",
    }
}

fn native_lane_name(value: crate::domain_installation::WorthQueryNativeFactLane) -> &'static str {
    match value {
        crate::domain_installation::WorthQueryNativeFactLane::Display => "display",
        crate::domain_installation::WorthQueryNativeFactLane::Derived => "derived",
    }
}
