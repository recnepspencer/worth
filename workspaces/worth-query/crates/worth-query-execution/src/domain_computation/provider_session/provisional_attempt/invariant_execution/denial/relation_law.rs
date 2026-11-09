//! Borrowed law evidence from the original native refusal, never a second verdict.
use worth_relational::facade::config::CascadeDeletePolicy;
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::schema::ContractId;
use worth_relational::facade::schema::{
    EndpointDeletionIntegrityMode, SymmetryMode, UniquenessScope,
};
use worth_relational::facade::transactions::{
    ConflictClass, EntityReference, InvariantViolationFields, RelationCardinalityBoundary,
    TransactionCommitError,
};

/// The relation law and typed identities that refused a proposed commit.
///
/// These values identify the original violation. They grant no graph access or
/// mutation authority. A created endpoint names an entity in the refused
/// proposal, not a published entity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WorthQueryRelationLawDenial<'a> {
    /// A member has too few or too many physical incident edges on this side.
    Cardinality {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        member: &'a EntityReference,
        boundary: RelationCardinalityBoundary,
        count: usize,
        limit: u64,
    },
    /// A deleted endpoint still has live incident edges under this deletion policy.
    EndpointDeletion {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        entity_id: &'a EntityId,
        remaining_relation_endpoint_count: usize,
        mode: EndpointDeletionIntegrityMode,
        cascade_delete_policy: Option<CascadeDeletePolicy>,
    },
    /// The relation's endpoint policy forbids this original self edge.
    SelfEdge {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        target: &'a EntityReference,
        self_edge: bool,
    },
    /// A source has too few distinct reachable targets of the declared kinds.
    ConnectivityMinimum {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        reachable_target_count: usize,
        minimum_reachable_targets: u32,
    },
    /// These endpoints belong to different native partitions.
    PartitionIsolation {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        target: &'a EntityReference,
    },
    /// A surviving edge has no required reverse edge on the same relation.
    Symmetry {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        target: &'a EntityReference,
        mode: SymmetryMode,
    },
    /// Multiple physical records describe the same edge in the declared scope.
    Uniqueness {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        target: &'a EntityReference,
        scope: UniquenessScope,
        count: usize,
    },
    /// This source-to-target edge closes a directed cycle in the proposed graph.
    Acyclicity {
        contract_id: &'a ContractId,
        relation_kind_id: KindId,
        source: &'a EntityReference,
        target: &'a EntityReference,
    },
}

impl<'a> WorthQueryRelationLawDenial<'a> {
    pub(crate) fn from_fields(fields: &'a InvariantViolationFields) -> Option<Self> {
        match fields {
            InvariantViolationFields::RelationCardinalityEndpoint {
                contract_id,
                relation_kind_id,
                entity_id,
                boundary,
                count,
                limit,
            } => Some(Self::Cardinality {
                contract_id,
                relation_kind_id: *relation_kind_id,
                member: entity_id,
                boundary: *boundary,
                count: *count,
                limit: *limit,
            }),
            InvariantViolationFields::RelationEndpointKindSelfEdge {
                contract_id,
                relation_kind_id,
                source,
                target,
                self_edge,
            } => Some(Self::SelfEdge {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                target,
                self_edge: *self_edge,
            }),
            InvariantViolationFields::ConnectivityMinimum {
                contract_id,
                relation_kind_id,
                source,
                reachable_target_count,
                minimum_reachable_targets,
            } => Some(Self::ConnectivityMinimum {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                reachable_target_count: *reachable_target_count,
                minimum_reachable_targets: *minimum_reachable_targets,
            }),
            InvariantViolationFields::PartitionIsolation {
                contract_id,
                relation_kind_id,
                source,
                target,
                ..
            } => Some(Self::PartitionIsolation {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                target,
            }),
            InvariantViolationFields::RelationSymmetry {
                contract_id,
                relation_kind_id,
                source,
                target,
                mode,
            } => Some(Self::Symmetry {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                target,
                mode: *mode,
            }),
            InvariantViolationFields::RelationUniqueness {
                contract_id,
                relation_kind_id,
                source,
                target,
                scope,
                count,
            } => Some(Self::Uniqueness {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                target,
                scope: *scope,
                count: *count,
            }),
            InvariantViolationFields::Acyclicity {
                contract_id,
                relation_kind_id,
                source,
                target,
            } => Some(Self::Acyclicity {
                contract_id,
                relation_kind_id: *relation_kind_id,
                source,
                target,
            }),
            InvariantViolationFields::RelationEndpointDeletionIntegrity {
                contract_id,
                relation_kind_id,
                entity_id,
                remaining_relation_endpoint_count,
                mode,
                cascade_delete_policy,
            } => Some(Self::EndpointDeletion {
                contract_id,
                relation_kind_id: *relation_kind_id,
                entity_id,
                remaining_relation_endpoint_count: *remaining_relation_endpoint_count,
                mode: *mode,
                cascade_delete_policy: *cascade_delete_policy,
            }),
            InvariantViolationFields::None
            | InvariantViolationFields::MergedIntentLimit { .. }
            | InvariantViolationFields::SnapshotEntityLimit { .. }
            | InvariantViolationFields::UniqueEntityField { .. }
            | InvariantViolationFields::SidecarConsistency { .. }
            | InvariantViolationFields::RelationEndpointKindMismatch { .. }
            | InvariantViolationFields::RelationEndpointKindCrossContext { .. }
            | InvariantViolationFields::RelationCardinalityPair { .. }
            | InvariantViolationFields::StorageInconsistency { .. }
            | InvariantViolationFields::CustomInvariantFailure { .. }
            | InvariantViolationFields::CustomInvariantViolation { .. }
            | InvariantViolationFields::DerivedRelationGraph { .. } => None,
        }
    }

    pub(crate) fn from_commit_error(error: &'a TransactionCommitError) -> Option<Self> {
        match error {
            TransactionCommitError::Conflict { error, .. } => {
                let ConflictClass::InvariantViolation { fields, .. } = &error.class else {
                    return None;
                };
                Self::from_fields(fields)
            }
            TransactionCommitError::Publication { error, .. } => {
                Self::from_fields(&error.invariant_violation()?.fields)
            }
            _ => None,
        }
    }
}
