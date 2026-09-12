use crate::diagnostics::data::RelationalDiagnosticValue;

use super::{violation_diagnostic_object, InvariantViolationDiagnosticProjection};

mod relation_projection;
mod typed_value_helpers;

impl InvariantViolationDiagnosticProjection<'_> {
    pub(in crate::validation::invariant_authority) fn to_diagnostic_value(
        &self,
    ) -> RelationalDiagnosticValue {
        match self {
            Self::None => violation_diagnostic_object(
                "none",
                std::iter::empty::<(String, RelationalDiagnosticValue)>(),
            ),
            Self::MergedIntentLimit {
                merged_intent_count,
                limit,
            } => violation_diagnostic_object(
                "merged_intent_limit",
                [
                    (
                        "merged_intent_count",
                        RelationalDiagnosticValue::unsigned(*merged_intent_count),
                    ),
                    ("limit", RelationalDiagnosticValue::unsigned(*limit)),
                ],
            ),
            Self::SnapshotEntityLimit {
                version_id,
                visible_entities,
                limit,
            } => violation_diagnostic_object(
                "snapshot_entity_limit",
                [
                    (
                        "version_id",
                        RelationalDiagnosticValue::VersionId(*version_id),
                    ),
                    (
                        "visible_entities",
                        RelationalDiagnosticValue::unsigned(*visible_entities),
                    ),
                    ("limit", RelationalDiagnosticValue::unsigned(*limit)),
                ],
            ),
            Self::UniqueEntityField { aspect_field } => violation_diagnostic_object(
                "unique_entity_field",
                [("aspect_field", aspect_field.to_diagnostic_value())],
            ),
            Self::SidecarConsistency {
                partition_id,
                slot,
                missing_label,
            } => violation_diagnostic_object(
                "sidecar_consistency",
                [
                    (
                        "partition_id",
                        RelationalDiagnosticValue::PartitionId(*partition_id),
                    ),
                    ("slot", RelationalDiagnosticValue::unsigned(*slot)),
                    (
                        "missing_label",
                        RelationalDiagnosticValue::string(*missing_label),
                    ),
                ],
            ),
            Self::RelationEndpointKindMismatch {
                contract_id,
                relation_kind_id,
                source,
                target,
                source_kind_id,
                target_kind_id,
                boundary,
            } => relation_projection::endpoint_kind_mismatch(
                contract_id,
                *relation_kind_id,
                source,
                target,
                *source_kind_id,
                *target_kind_id,
                *boundary,
            ),
            Self::RelationEndpointKindSelfEdge {
                contract_id,
                relation_kind_id,
                source,
                target,
                self_edge,
            } => relation_projection::endpoint_kind_self_edge(
                contract_id,
                *relation_kind_id,
                source,
                target,
                *self_edge,
            ),
            Self::RelationEndpointKindCrossContext {
                contract_id,
                relation_kind_id,
                source_partition_id,
                target_partition_id,
            } => relation_projection::endpoint_kind_cross_context(
                contract_id,
                *relation_kind_id,
                *source_partition_id,
                *target_partition_id,
            ),
            Self::RelationCardinalityEndpoint {
                contract_id,
                relation_kind_id,
                entity_id,
                boundary,
                count,
                limit,
            } => relation_projection::cardinality_endpoint(
                contract_id,
                *relation_kind_id,
                entity_id,
                *boundary,
                *count,
                *limit,
            ),
            Self::RelationCardinalityPair {
                contract_id,
                relation_kind_id,
                source,
                target,
                count,
                limit,
            } => relation_projection::cardinality_pair(
                contract_id,
                *relation_kind_id,
                source,
                target,
                *count,
                *limit,
            ),
            Self::RelationUniqueness {
                contract_id,
                relation_kind_id,
                scope,
                source,
                target,
                count,
            } => relation_projection::uniqueness(
                contract_id,
                *relation_kind_id,
                *scope,
                source,
                target,
                *count,
            ),
            Self::RelationSymmetry {
                contract_id,
                relation_kind_id,
                source,
                target,
                mode,
            } => {
                relation_projection::symmetry(contract_id, *relation_kind_id, source, target, *mode)
            }
            Self::RelationEndpointDeletionIntegrity {
                contract_id,
                relation_kind_id,
                entity_id,
                remaining_relation_endpoint_count,
                mode,
                cascade_delete_policy,
            } => relation_projection::endpoint_deletion_integrity(
                contract_id,
                *relation_kind_id,
                *entity_id,
                *remaining_relation_endpoint_count,
                *mode,
                *cascade_delete_policy,
            ),
            Self::StorageInconsistency {
                entity_id,
                partition_id,
                slot,
                field,
                missing_label,
                scan,
                lookup,
                failure,
            } => violation_diagnostic_object(
                "storage_inconsistency",
                [
                    (
                        "entity_id",
                        RelationalDiagnosticValue::optional(
                            entity_id.map(RelationalDiagnosticValue::EntityId),
                        ),
                    ),
                    (
                        "partition_id",
                        RelationalDiagnosticValue::optional(
                            partition_id.map(RelationalDiagnosticValue::PartitionId),
                        ),
                    ),
                    (
                        "slot",
                        RelationalDiagnosticValue::optional(
                            slot.map(RelationalDiagnosticValue::unsigned),
                        ),
                    ),
                    (
                        "field",
                        RelationalDiagnosticValue::optional(
                            field.map(|field| RelationalDiagnosticValue::FieldKey(field.clone())),
                        ),
                    ),
                    (
                        "missing_label",
                        typed_value_helpers::optional_label(*missing_label),
                    ),
                    (
                        "scan",
                        typed_value_helpers::optional_label(
                            scan.map(|value| value.diagnostic_label()),
                        ),
                    ),
                    (
                        "lookup",
                        typed_value_helpers::optional_label(
                            lookup.map(|value| value.diagnostic_label()),
                        ),
                    ),
                    (
                        "failure",
                        typed_value_helpers::optional_label(
                            failure.map(|value| value.diagnostic_label()),
                        ),
                    ),
                ],
            ),
            Self::RelationIntegrityScopeBudgetExceeded {
                limit_name,
                limit,
                observed,
                relation_kind_count,
                touched_entity_count,
                deleted_entity_count,
                scanned_relation_count,
                planned_edge_count,
            } => violation_diagnostic_object(
                "relation_integrity_scope_budget_exceeded",
                [
                    ("limit_name", RelationalDiagnosticValue::string(*limit_name)),
                    ("limit", RelationalDiagnosticValue::unsigned(*limit)),
                    ("observed", RelationalDiagnosticValue::unsigned(*observed)),
                    (
                        "relation_kind_count",
                        RelationalDiagnosticValue::unsigned(*relation_kind_count),
                    ),
                    (
                        "touched_entity_count",
                        RelationalDiagnosticValue::unsigned(*touched_entity_count),
                    ),
                    (
                        "deleted_entity_count",
                        RelationalDiagnosticValue::unsigned(*deleted_entity_count),
                    ),
                    (
                        "scanned_relation_count",
                        RelationalDiagnosticValue::unsigned(*scanned_relation_count),
                    ),
                    (
                        "planned_edge_count",
                        RelationalDiagnosticValue::unsigned(*planned_edge_count),
                    ),
                ],
            ),
            Self::CustomInvariantFailure {
                identity,
                phase,
                failure,
                detail,
            } => violation_diagnostic_object(
                "custom_invariant_failure",
                [
                    (
                        "identity",
                        typed_value_helpers::custom_invariant_identity_diagnostic_value(identity),
                    ),
                    (
                        "phase",
                        RelationalDiagnosticValue::string(phase.diagnostic_label()),
                    ),
                    (
                        "failure",
                        RelationalDiagnosticValue::string(failure.diagnostic_label()),
                    ),
                    ("detail", RelationalDiagnosticValue::string(*detail)),
                ],
            ),
            Self::CustomInvariantViolation { identity } => violation_diagnostic_object(
                "custom_invariant_violation",
                [(
                    "identity",
                    typed_value_helpers::custom_invariant_semantic_identity_diagnostic_value(
                        identity,
                    ),
                )],
            ),
            Self::PartitionIsolation {
                contract_id,
                relation_kind_id,
                relation_id,
                source_partition_id,
                target_partition_id,
            } => relation_projection::partition_isolation(
                contract_id,
                *relation_kind_id,
                *relation_id,
                *source_partition_id,
                *target_partition_id,
            ),
            Self::Acyclicity {
                contract_id,
                relation_kind_id,
                source,
                target,
            } => relation_projection::acyclicity(contract_id, *relation_kind_id, source, target),
            Self::ConnectivityMinimum {
                contract_id,
                relation_kind_id,
                source,
                reachable_target_count,
                minimum_reachable_targets,
            } => relation_projection::connectivity_minimum(
                contract_id,
                *relation_kind_id,
                source,
                *reachable_target_count,
                *minimum_reachable_targets,
            ),
        }
    }
}
