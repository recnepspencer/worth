use crate::config::data::CascadeDeletePolicy;
use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId};
use crate::schema::data::{
    ContractId, EndpointDeletionIntegrityMode, SymmetryMode, UniquenessScope,
};
use crate::transactions::data::EntityReference;
use crate::validation::data::{RelationCardinalityBoundary, RelationEndpointBoundary};

use super::typed_value_helpers::{
    entity_reference_diagnostic_value, optional_label, relation_cardinality_boundary_label,
    relation_endpoint_boundary_label,
};
use crate::validation::invariant_authority::invariant_violation_diagnostic_projection::violation_diagnostic_object;

pub(super) fn endpoint_kind_mismatch(
    contract_id: &ContractId,
    relation_kind_id: KindId,
    source: &EntityReference,
    target: &EntityReference,
    source_kind_id: KindId,
    target_kind_id: KindId,
    boundary: RelationEndpointBoundary,
) -> RelationalDiagnosticValue {
    violation_diagnostic_object(
        "relation_endpoint_kind_mismatch",
        [
            contract_field(contract_id),
            kind_field(relation_kind_id),
            ("source", entity_reference_diagnostic_value(source)),
            ("target", entity_reference_diagnostic_value(target)),
            (
                "source_kind_id",
                RelationalDiagnosticValue::KindId(source_kind_id),
            ),
            (
                "target_kind_id",
                RelationalDiagnosticValue::KindId(target_kind_id),
            ),
            (
                "boundary",
                RelationalDiagnosticValue::string(relation_endpoint_boundary_label(boundary)),
            ),
        ],
    )
}

pub(super) fn endpoint_kind_self_edge(
    contract_id: &ContractId,
    relation_kind_id: KindId,
    source: &EntityReference,
    target: &EntityReference,
    self_edge: bool,
) -> RelationalDiagnosticValue {
    violation_diagnostic_object(
        "relation_endpoint_kind_self_edge",
        [
            contract_field(contract_id),
            kind_field(relation_kind_id),
            ("source", entity_reference_diagnostic_value(source)),
            ("target", entity_reference_diagnostic_value(target)),
            ("self_edge", RelationalDiagnosticValue::Bool(self_edge)),
        ],
    )
}

pub(super) fn endpoint_kind_cross_context(
    contract_id: &ContractId,
    relation_kind_id: KindId,
    source_partition_id: PartitionId,
    target_partition_id: PartitionId,
) -> RelationalDiagnosticValue {
    violation_diagnostic_object(
        "relation_endpoint_kind_cross_context",
        [
            contract_field(contract_id),
            kind_field(relation_kind_id),
            (
                "source_partition_id",
                RelationalDiagnosticValue::PartitionId(source_partition_id),
            ),
            (
                "target_partition_id",
                RelationalDiagnosticValue::PartitionId(target_partition_id),
            ),
        ],
    )
}

pub(super) fn cardinality_endpoint(
    contract_id: &ContractId,
    relation_kind_id: KindId,
    entity_id: &EntityReference,
    boundary: RelationCardinalityBoundary,
    count: usize,
    limit: u64,
) -> RelationalDiagnosticValue {
    violation_diagnostic_object(
        "relation_cardinality_endpoint",
        [
            contract_field(contract_id),
            kind_field(relation_kind_id),
            ("entity", entity_reference_diagnostic_value(entity_id)),
            (
                "boundary",
                RelationalDiagnosticValue::string(relation_cardinality_boundary_label(boundary)),
            ),
            ("count", RelationalDiagnosticValue::unsigned(count)),
            ("limit", RelationalDiagnosticValue::Unsigned(limit)),
        ],
    )
}

pub(super) fn cardinality_pair(
    contract_id: &ContractId,
    relation_kind_id: KindId,
    source: &EntityReference,
    target: &EntityReference,
    count: usize,
    limit: u64,
) -> RelationalDiagnosticValue {
    violation_diagnostic_object(
        "relation_cardinality_pair",
        [
            contract_field(contract_id),
            kind_field(relation_kind_id),
            ("source", entity_reference_diagnostic_value(source)),
            ("target", entity_reference_diagnostic_value(target)),
            ("count", RelationalDiagnosticValue::unsigned(count)),
            ("limit", RelationalDiagnosticValue::Unsigned(limit)),
        ],
    )
}

pub(super) fn uniqueness(
    contract_id: &ContractId,
    relation_kind_id: KindId,
    scope: UniquenessScope,
    source: &EntityReference,
    target: &EntityReference,
    count: usize,
) -> RelationalDiagnosticValue {
    violation_diagnostic_object(
        "relation_uniqueness",
        [
            contract_field(contract_id),
            kind_field(relation_kind_id),
            (
                "scope",
                RelationalDiagnosticValue::string(uniqueness_scope_label(scope)),
            ),
            ("source", entity_reference_diagnostic_value(source)),
            ("target", entity_reference_diagnostic_value(target)),
            ("count", RelationalDiagnosticValue::unsigned(count)),
        ],
    )
}

pub(super) fn symmetry(
    contract_id: &ContractId,
    relation_kind_id: KindId,
    source: &EntityReference,
    target: &EntityReference,
    mode: SymmetryMode,
) -> RelationalDiagnosticValue {
    violation_diagnostic_object(
        "relation_symmetry",
        [
            contract_field(contract_id),
            kind_field(relation_kind_id),
            ("source", entity_reference_diagnostic_value(source)),
            ("target", entity_reference_diagnostic_value(target)),
            (
                "mode",
                RelationalDiagnosticValue::string(symmetry_mode_label(mode)),
            ),
        ],
    )
}

pub(super) fn endpoint_deletion_integrity(
    contract_id: &ContractId,
    relation_kind_id: KindId,
    entity_id: EntityId,
    remaining_relation_endpoint_count: usize,
    mode: EndpointDeletionIntegrityMode,
    cascade_delete_policy: Option<CascadeDeletePolicy>,
) -> RelationalDiagnosticValue {
    violation_diagnostic_object(
        "relation_endpoint_deletion_integrity",
        [
            contract_field(contract_id),
            kind_field(relation_kind_id),
            ("entity_id", RelationalDiagnosticValue::EntityId(entity_id)),
            (
                "remaining_relation_endpoint_count",
                RelationalDiagnosticValue::unsigned(remaining_relation_endpoint_count),
            ),
            (
                "mode",
                RelationalDiagnosticValue::string(endpoint_deletion_integrity_mode_label(mode)),
            ),
            (
                "cascade_delete_policy",
                optional_label(cascade_delete_policy.map(cascade_delete_policy_label)),
            ),
        ],
    )
}

pub(super) fn partition_isolation(
    contract_id: &ContractId,
    relation_kind_id: KindId,
    relation_id: Option<RelationId>,
    source_partition_id: PartitionId,
    target_partition_id: PartitionId,
    source: &EntityReference,
    target: &EntityReference,
) -> RelationalDiagnosticValue {
    violation_diagnostic_object(
        "partition_isolation",
        [
            ("source", entity_reference_diagnostic_value(source)),
            ("target", entity_reference_diagnostic_value(target)),
            contract_field(contract_id),
            kind_field(relation_kind_id),
            (
                "relation_id",
                RelationalDiagnosticValue::optional(
                    relation_id.map(RelationalDiagnosticValue::RelationId),
                ),
            ),
            (
                "source_partition_id",
                RelationalDiagnosticValue::PartitionId(source_partition_id),
            ),
            (
                "target_partition_id",
                RelationalDiagnosticValue::PartitionId(target_partition_id),
            ),
        ],
    )
}

pub(super) fn acyclicity(
    contract_id: &ContractId,
    relation_kind_id: KindId,
    source: &EntityReference,
    target: &EntityReference,
) -> RelationalDiagnosticValue {
    violation_diagnostic_object(
        "acyclicity",
        [
            contract_field(contract_id),
            kind_field(relation_kind_id),
            ("source", entity_reference_diagnostic_value(source)),
            ("target", entity_reference_diagnostic_value(target)),
        ],
    )
}

pub(super) fn connectivity_minimum(
    contract_id: &ContractId,
    relation_kind_id: KindId,
    source: &EntityReference,
    reachable_target_count: usize,
    minimum_reachable_targets: u32,
) -> RelationalDiagnosticValue {
    violation_diagnostic_object(
        "connectivity_minimum",
        [
            contract_field(contract_id),
            kind_field(relation_kind_id),
            ("source", entity_reference_diagnostic_value(source)),
            (
                "reachable_target_count",
                RelationalDiagnosticValue::unsigned(reachable_target_count),
            ),
            (
                "minimum_reachable_targets",
                RelationalDiagnosticValue::Unsigned(u64::from(minimum_reachable_targets)),
            ),
        ],
    )
}

fn contract_field(contract_id: &ContractId) -> (&'static str, RelationalDiagnosticValue) {
    (
        "contract_id",
        RelationalDiagnosticValue::ContractId(contract_id.clone()),
    )
}

fn kind_field(relation_kind_id: KindId) -> (&'static str, RelationalDiagnosticValue) {
    (
        "relation_kind_id",
        RelationalDiagnosticValue::KindId(relation_kind_id),
    )
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
        SymmetryMode::InverseProhibited => "inverse_prohibited",
        SymmetryMode::PairedTwinRequired => "paired_twin_required",
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

fn cascade_delete_policy_label(policy: CascadeDeletePolicy) -> &'static str {
    match policy {
        CascadeDeletePolicy::RetainDanglingForAudit => "retain_dangling_for_audit",
        CascadeDeletePolicy::CascadeDeleteRelations => "cascade_delete_relations",
    }
}
