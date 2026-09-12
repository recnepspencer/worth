use crate::validation::data::InvariantViolationFields;

use super::{AspectFieldDiagnosticProjection, InvariantViolationDiagnosticProjection};

impl<'a> InvariantViolationDiagnosticProjection<'a> {
    pub(in crate::validation::invariant_authority) fn from_fields(
        fields: &'a InvariantViolationFields,
    ) -> Self {
        match fields {
            InvariantViolationFields::None => Self::None,
            InvariantViolationFields::MergedIntentLimit {
                merged_intent_count,
                limit,
            } => Self::MergedIntentLimit {
                merged_intent_count: *merged_intent_count,
                limit: *limit,
            },
            InvariantViolationFields::SnapshotEntityLimit {
                version_id,
                visible_entities,
                limit,
            } => Self::SnapshotEntityLimit {
                version_id: *version_id,
                visible_entities: *visible_entities,
                limit: *limit,
            },
            InvariantViolationFields::UniqueEntityField {
                field_locator,
                value,
            } => Self::UniqueEntityField {
                aspect_field: AspectFieldDiagnosticProjection::new(
                    field_locator.aspect().aspect_key(),
                    field_locator.field_path().clone(),
                    value,
                ),
            },
            InvariantViolationFields::SidecarConsistency {
                partition_id,
                slot,
                missing_label,
            } => Self::SidecarConsistency {
                partition_id: *partition_id,
                slot: *slot,
                missing_label,
            },
            InvariantViolationFields::RelationEndpointKindMismatch {
                contract_id,
                relation_kind_id,
                source,
                target,
                source_kind_id,
                target_kind_id,
                boundary,
            } => Self::RelationEndpointKindMismatch {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                target,
                source_kind_id: *source_kind_id,
                target_kind_id: *target_kind_id,
                boundary: *boundary,
            },
            InvariantViolationFields::RelationEndpointKindSelfEdge {
                contract_id,
                relation_kind_id,
                source,
                target,
                self_edge,
            } => Self::RelationEndpointKindSelfEdge {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                target,
                self_edge: *self_edge,
            },
            InvariantViolationFields::RelationEndpointKindCrossContext {
                contract_id,
                relation_kind_id,
                source_partition_id,
                target_partition_id,
            } => Self::RelationEndpointKindCrossContext {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source_partition_id: *source_partition_id,
                target_partition_id: *target_partition_id,
            },
            InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id,
                relation_kind_id,
                entity_id,
                boundary,
                count,
                limit,
            } => Self::RelationCardinalityEndpoint {
                contract_id,
                relation_kind_id: *relation_kind_id,
                entity_id,
                boundary: *boundary,
                count: *count,
                limit: *limit,
            },
            InvariantViolationFields::RelationCardinalityPair {
                contract_id,
                relation_kind_id,
                source,
                target,
                count,
                limit,
            } => Self::RelationCardinalityPair {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                target,
                count: *count,
                limit: *limit,
            },
            InvariantViolationFields::RelationUniqueness {
                contract_id,
                relation_kind_id,
                scope,
                source,
                target,
                count,
            } => Self::RelationUniqueness {
                contract_id,
                relation_kind_id: *relation_kind_id,
                scope: *scope,
                source,
                target,
                count: *count,
            },
            InvariantViolationFields::RelationSymmetry {
                contract_id,
                relation_kind_id,
                source,
                target,
                mode,
            } => Self::RelationSymmetry {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                target,
                mode: *mode,
            },
            InvariantViolationFields::RelationEndpointDeletionIntegrity {
                contract_id,
                relation_kind_id,
                entity_id,
                remaining_relation_endpoint_count,
                mode,
                cascade_delete_policy,
            } => Self::RelationEndpointDeletionIntegrity {
                contract_id,
                relation_kind_id: *relation_kind_id,
                entity_id: *entity_id,
                remaining_relation_endpoint_count: *remaining_relation_endpoint_count,
                mode: *mode,
                cascade_delete_policy: *cascade_delete_policy,
            },
            InvariantViolationFields::StorageInconsistency {
                entity_id,
                partition_id,
                slot,
                field,
                missing_label,
                scan,
                lookup,
                failure,
            } => Self::StorageInconsistency {
                entity_id: *entity_id,
                partition_id: *partition_id,
                slot: *slot,
                field: field.as_ref(),
                missing_label: missing_label.as_deref(),
                scan: *scan,
                lookup: *lookup,
                failure: *failure,
            },
            InvariantViolationFields::RelationIntegrityScopeBudgetExceeded {
                limit_name,
                limit,
                observed,
                relation_kind_count,
                touched_entity_count,
                deleted_entity_count,
                scanned_relation_count,
                planned_edge_count,
            } => Self::RelationIntegrityScopeBudgetExceeded {
                limit_name,
                limit: *limit,
                observed: *observed,
                relation_kind_count: *relation_kind_count,
                touched_entity_count: *touched_entity_count,
                deleted_entity_count: *deleted_entity_count,
                scanned_relation_count: *scanned_relation_count,
                planned_edge_count: *planned_edge_count,
            },
            InvariantViolationFields::CustomInvariantFailure {
                identity,
                phase,
                failure,
                detail,
            } => Self::CustomInvariantFailure {
                identity,
                phase: *phase,
                failure: *failure,
                detail,
            },
            InvariantViolationFields::CustomInvariantViolation { identity } => {
                Self::CustomInvariantViolation { identity }
            }
            InvariantViolationFields::PartitionIsolation {
                contract_id,
                relation_kind_id,
                relation_id,
                source_partition_id,
                target_partition_id,
            } => Self::PartitionIsolation {
                contract_id,
                relation_kind_id: *relation_kind_id,
                relation_id: *relation_id,
                source_partition_id: *source_partition_id,
                target_partition_id: *target_partition_id,
            },
            InvariantViolationFields::Acyclicity {
                contract_id,
                relation_kind_id,
                source,
                target,
            } => Self::Acyclicity {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                target,
            },
            InvariantViolationFields::ConnectivityMinimum {
                contract_id,
                relation_kind_id,
                source,
                reachable_target_count,
                minimum_reachable_targets,
            } => Self::ConnectivityMinimum {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                reachable_target_count: *reachable_target_count,
                minimum_reachable_targets: *minimum_reachable_targets,
            },
        }
    }
}
