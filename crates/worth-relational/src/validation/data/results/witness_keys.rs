use crate::aspect_wire::{encode_aspect_field_locator, encode_length_prefixed_aspect_value};
use crate::diagnostics::data::DiagnosticCode;
use crate::schema::data::{EndpointDeletionIntegrityMode, SymmetryMode, UniquenessScope};
use crate::validation::data::InvariantWitnessKey;

use super::{InvariantViolationFields, RelationCardinalityBoundary, RelationEndpointBoundary};

pub(super) fn invariant_violation_witness_key(
    code: DiagnosticCode,
    fields: &InvariantViolationFields,
) -> InvariantWitnessKey {
    let key = match fields {
        InvariantViolationFields::None => format!("none:{code:?}"),
        InvariantViolationFields::MergedIntentLimit {
            merged_intent_count,
            limit,
        } => format!("merged_intents:{merged_intent_count}:{limit}"),
        InvariantViolationFields::SnapshotEntityLimit {
            version_id, limit, ..
        } => {
            format!("snapshot_entity_limit:{}:{limit}", version_id.as_u64())
        }
        InvariantViolationFields::UniqueEntityField {
            field_locator,
            value,
        } => return unique_entity_aspect_field_witness_key(code, field_locator, value),
        InvariantViolationFields::SidecarConsistency {
            partition_id,
            slot,
            missing_label,
        } => format!(
            "sidecar_consistency:{}:{slot}:{missing_label}",
            partition_id.as_u32()
        ),
        InvariantViolationFields::RelationEndpointKindMismatch {
            relation_kind_id,
            source,
            target,
            boundary,
            ..
        } => format!(
            "endpoint_kind_mismatch:{}:{source:?}:{target:?}:{}",
            relation_kind_id.as_u32(),
            relation_endpoint_boundary_label(*boundary)
        ),
        InvariantViolationFields::RelationEndpointKindSelfEdge {
            relation_kind_id,
            source,
            target,
            ..
        } => format!(
            "endpoint_kind_self_edge:{}:{source:?}:{target:?}",
            relation_kind_id.as_u32()
        ),
        InvariantViolationFields::RelationEndpointKindCrossContext {
            relation_kind_id,
            source_partition_id,
            target_partition_id,
            ..
        } => format!(
            "endpoint_kind_cross_context:{}:{}:{}",
            relation_kind_id.as_u32(),
            source_partition_id.as_u32(),
            target_partition_id.as_u32()
        ),
        InvariantViolationFields::RelationCardinalityEndpoint {
            relation_kind_id,
            entity_id,
            boundary,
            ..
        } => format!(
            "cardinality_endpoint:{}:{entity_id:?}:{}",
            relation_kind_id.as_u32(),
            relation_cardinality_boundary_label(*boundary)
        ),
        InvariantViolationFields::RelationCardinalityPair {
            relation_kind_id,
            source,
            target,
            ..
        } => format!(
            "cardinality_pair:{}:{source:?}:{target:?}",
            relation_kind_id.as_u32()
        ),
        InvariantViolationFields::RelationUniqueness {
            relation_kind_id,
            scope,
            source,
            target,
            ..
        } => format!(
            "uniqueness:{}:{source:?}:{target:?}:{}",
            relation_kind_id.as_u32(),
            uniqueness_scope_label(*scope)
        ),
        InvariantViolationFields::RelationSymmetry {
            relation_kind_id,
            source,
            target,
            mode,
            ..
        } => format!(
            "symmetry:{}:{source:?}:{target:?}:{}",
            relation_kind_id.as_u32(),
            symmetry_mode_label(*mode)
        ),
        InvariantViolationFields::RelationEndpointDeletionIntegrity {
            relation_kind_id,
            entity_id,
            mode,
            ..
        } => format!(
            "endpoint_deletion:{}:{entity_id:?}:{}",
            relation_kind_id.as_u32(),
            endpoint_deletion_integrity_mode_label(*mode)
        ),
        InvariantViolationFields::StorageInconsistency {
            entity_id,
            partition_id,
            slot,
            field,
            scan,
            lookup,
            failure,
            ..
        } => {
            format!(
                "storage_inconsistency:{entity_id:?}:{partition_id:?}:{slot:?}:{}:{}:{}:{}",
                field.as_ref().map(|field| field.as_str()).unwrap_or("none"),
                scan.map(|value| value.diagnostic_label()).unwrap_or("none"),
                lookup
                    .map(|value| value.diagnostic_label())
                    .unwrap_or("none"),
                failure
                    .map(|value| value.diagnostic_label())
                    .unwrap_or("none")
            )
        }
        InvariantViolationFields::RelationIntegrityScopeBudgetExceeded {
            limit_name,
            observed,
            ..
        } => format!("scope_budget:{limit_name}:{observed}"),
        InvariantViolationFields::CustomInvariantFailure {
            identity,
            phase,
            failure,
            ..
        } => format!(
            "custom_failure:{}:{}.{}:{}:{}",
            identity.semantic_identity().rule_id.as_str(),
            identity.semantic_identity().semantic_version.major,
            identity.semantic_identity().semantic_version.minor,
            phase.diagnostic_label(),
            failure.diagnostic_label()
        ),
        InvariantViolationFields::CustomInvariantViolation { identity } => format!(
            "custom_violation:{}:{}.{}",
            identity.rule_id.as_str(),
            identity.semantic_version.major,
            identity.semantic_version.minor,
        ),
        InvariantViolationFields::PartitionIsolation {
            relation_kind_id,
            relation_id,
            source_partition_id,
            target_partition_id,
            ..
        } => format!(
            "partition_isolation:{}:{relation_id:?}:{}:{}",
            relation_kind_id.as_u32(),
            source_partition_id.as_u32(),
            target_partition_id.as_u32()
        ),
        InvariantViolationFields::Acyclicity {
            relation_kind_id,
            source,
            target,
            ..
        } => format!(
            "acyclicity:{}:{source:?}:{target:?}",
            relation_kind_id.as_u32()
        ),
        InvariantViolationFields::ConnectivityMinimum {
            relation_kind_id,
            source,
            ..
        } => format!(
            "connectivity_minimum:{}:{source:?}",
            relation_kind_id.as_u32()
        ),
    };
    InvariantWitnessKey::new(key)
}

fn unique_entity_aspect_field_witness_key(
    code: DiagnosticCode,
    field_locator: &worth_foundational::facade::AspectFieldLocator,
    value: &worth_foundational::facade::AspectValue,
) -> InvariantWitnessKey {
    let field_locator_canonical_bytes = encode_aspect_field_locator(field_locator);
    let value_canonical_bytes = canonical_aspect_value_witness_basis(value);
    let key = format!(
        "unique_entity_aspect_field:{code:?}:{}:{}",
        hex_bytes(&field_locator_canonical_bytes),
        hex_bytes(&value_canonical_bytes)
    );
    InvariantWitnessKey::unique_entity_aspect_field(
        key,
        field_locator.clone(),
        value.clone(),
        field_locator_canonical_bytes,
        value_canonical_bytes,
    )
}

fn canonical_aspect_value_witness_basis(
    value: &worth_foundational::facade::AspectValue,
) -> Vec<u8> {
    let mut bytes = Vec::new();
    encode_length_prefixed_aspect_value(&mut bytes, value);
    bytes
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

fn relation_endpoint_boundary_label(boundary: RelationEndpointBoundary) -> &'static str {
    match boundary {
        RelationEndpointBoundary::Source => "source",
        RelationEndpointBoundary::Target => "target",
    }
}

fn relation_cardinality_boundary_label(boundary: RelationCardinalityBoundary) -> &'static str {
    match boundary {
        RelationCardinalityBoundary::Source => "source",
        RelationCardinalityBoundary::Target => "target",
        RelationCardinalityBoundary::Pair => "pair",
    }
}

fn uniqueness_scope_label(scope: UniquenessScope) -> &'static str {
    match scope {
        UniquenessScope::DirectedSemanticEdge => "directed",
        UniquenessScope::NormalizedSymmetricEdge => "normalized",
    }
}

fn symmetry_mode_label(mode: SymmetryMode) -> &'static str {
    match mode {
        SymmetryMode::CanonicalUndirected => "canonical_undirected",
        SymmetryMode::PairedInverseRequired => "paired_inverse_required",
        SymmetryMode::PairedTwinRequired => "paired_twin_required",
        SymmetryMode::InverseProhibited => "inverse_prohibited",
    }
}

fn endpoint_deletion_integrity_mode_label(mode: EndpointDeletionIntegrityMode) -> &'static str {
    match mode {
        EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations => {
            "reject_delete_with_live_relations"
        }
        EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit => {
            "require_relation_deletion_in_same_commit"
        }
        EndpointDeletionIntegrityMode::RequireRelationRetirement => "require_relation_retirement",
    }
}
