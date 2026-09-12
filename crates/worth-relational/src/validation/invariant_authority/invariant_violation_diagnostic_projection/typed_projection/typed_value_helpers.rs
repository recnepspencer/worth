use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::transactions::data::EntityReference;
use crate::validation::data::{
    CustomInvariantFailureIdentity, RelationCardinalityBoundary, RelationEndpointBoundary,
};

pub(super) fn entity_reference_diagnostic_value(
    reference: &EntityReference,
) -> RelationalDiagnosticValue {
    match reference {
        EntityReference::Existing(entity_id) => RelationalDiagnosticValue::object([
            (
                "reference_kind",
                RelationalDiagnosticValue::string("existing"),
            ),
            ("entity_id", RelationalDiagnosticValue::EntityId(*entity_id)),
        ]),
        EntityReference::Created(created) => RelationalDiagnosticValue::object([
            (
                "reference_kind",
                RelationalDiagnosticValue::string("created"),
            ),
            (
                "partition_id",
                RelationalDiagnosticValue::PartitionId(created.partition_id),
            ),
            (
                "kind_id",
                RelationalDiagnosticValue::KindId(created.kind_id),
            ),
            (
                "client_key",
                RelationalDiagnosticValue::string(created.client_key.canonical_text().to_string()),
            ),
        ]),
    }
}

pub(super) fn custom_invariant_identity_diagnostic_value(
    identity: &CustomInvariantFailureIdentity,
) -> RelationalDiagnosticValue {
    let semantic_identity = identity.semantic_identity();
    RelationalDiagnosticValue::object([
        (
            "rule_id",
            RelationalDiagnosticValue::string(semantic_identity.rule_id.as_str()),
        ),
        (
            "semantic_version_major",
            RelationalDiagnosticValue::Unsigned(u64::from(
                semantic_identity.semantic_version.major,
            )),
        ),
        (
            "semantic_version_minor",
            RelationalDiagnosticValue::Unsigned(u64::from(
                semantic_identity.semantic_version.minor,
            )),
        ),
    ])
}

pub(super) fn custom_invariant_semantic_identity_diagnostic_value(
    identity: &crate::validation::data::CustomInvariantSemanticIdentity,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "rule_id",
            RelationalDiagnosticValue::string(identity.rule_id.as_str()),
        ),
        (
            "semantic_version_major",
            RelationalDiagnosticValue::Unsigned(u64::from(identity.semantic_version.major)),
        ),
        (
            "semantic_version_minor",
            RelationalDiagnosticValue::Unsigned(u64::from(identity.semantic_version.minor)),
        ),
    ])
}

pub(super) fn optional_label(label: Option<&str>) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(label.map(RelationalDiagnosticValue::string))
}

pub(super) fn relation_endpoint_boundary_label(boundary: RelationEndpointBoundary) -> &'static str {
    match boundary {
        RelationEndpointBoundary::Source => "source",
        RelationEndpointBoundary::Target => "target",
    }
}

pub(super) fn relation_cardinality_boundary_label(
    boundary: RelationCardinalityBoundary,
) -> &'static str {
    match boundary {
        RelationCardinalityBoundary::Source => "source",
        RelationCardinalityBoundary::Target => "target",
        RelationCardinalityBoundary::Pair => "pair",
    }
}
