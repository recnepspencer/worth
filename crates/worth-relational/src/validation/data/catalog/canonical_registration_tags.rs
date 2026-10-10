use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::schema::data::{
    AllowedCycleClass, ConnectivityMinimumEnforcement, DirectedTraversalKind,
    EndpointDeletionIntegrityMode, MinimumCardinalityEnforcement, PairMinimumSemantics,
    PartitionIsolationMode, SymmetryMode, UniquenessScope,
};
use crate::validation::data::{
    InvariantCostClass, InvariantExecutionPoint, InvariantFailureEffect, InvariantSemanticsClass,
    NativeInvariantRuleId, RecordKindTag,
};

pub(super) fn native_rule_id_tag(value: NativeInvariantRuleId) -> u8 {
    match value {
        NativeInvariantRuleId::LiveRecordRequiresSidecarEntity => 1,
        NativeInvariantRuleId::LiveRecordRequiresSidecarRelation => 2,
        NativeInvariantRuleId::MaxMergedIntents => 3,
        // Tag 4 belonged to the retired synthetic preparation quota rule.
        NativeInvariantRuleId::MaxSnapshotEntities => 5,
        NativeInvariantRuleId::UniqueEntityField => 6,
        NativeInvariantRuleId::EndpointKindContract => 7,
        NativeInvariantRuleId::CardinalityMaximumContract => 8,
        NativeInvariantRuleId::CardinalityMinimumContract => 9,
        NativeInvariantRuleId::UniquenessContract => 10,
        NativeInvariantRuleId::SymmetryContract => 11,
        NativeInvariantRuleId::EndpointDeletionIntegrityContract => 12,
        NativeInvariantRuleId::AcyclicityContract => 13,
        NativeInvariantRuleId::PartitionIsolationContract => 14,
        NativeInvariantRuleId::ConnectivityMinimumContract => 15,
    }
}

pub(super) fn record_kind_tag(value: &RecordKindTag) -> u8 {
    match value {
        RecordKindTag::Entity => 1,
        RecordKindTag::Relation => 2,
    }
}

pub(super) fn execution_point_tag(value: InvariantExecutionPoint) -> u8 {
    match value {
        InvariantExecutionPoint::MutationSensitive => 1,
        InvariantExecutionPoint::CommitBoundary => 2,
        InvariantExecutionPoint::SnapshotPublication => 3,
        InvariantExecutionPoint::CertificationBoundary => 4,
        InvariantExecutionPoint::HarnessAudit => 5,
        InvariantExecutionPoint::GraphComposition => 6,
    }
}

pub(super) fn failure_effect_tag(value: InvariantFailureEffect) -> u8 {
    match value {
        InvariantFailureEffect::BlockCommit => 1,
        InvariantFailureEffect::BlockPublication => 2,
        InvariantFailureEffect::AuditOnly => 3,
    }
}

pub(super) fn cost_class_tag(value: InvariantCostClass) -> u8 {
    match value {
        InvariantCostClass::Touched => 1,
        InvariantCostClass::Partition => 2,
        InvariantCostClass::Global => 3,
    }
}

pub(super) fn semantics_class_tag(value: InvariantSemanticsClass) -> u8 {
    match value {
        InvariantSemanticsClass::NativeAlwaysOn => 1,
        InvariantSemanticsClass::NativeSchemaLowered => 2,
        InvariantSemanticsClass::CustomStructural => 3,
    }
}

pub(super) fn cross_context_policy_tag(value: CrossContextPolicy) -> u8 {
    match value {
        CrossContextPolicy::AllowExplicit => 1,
        CrossContextPolicy::SchemaControlled => 2,
        CrossContextPolicy::Forbid => 3,
    }
}

pub(super) fn cascade_delete_policy_tag(value: CascadeDeletePolicy) -> u8 {
    match value {
        CascadeDeletePolicy::RetainDanglingForAudit => 1,
        CascadeDeletePolicy::CascadeDeleteRelations => 2,
    }
}

pub(super) fn pair_minimum_semantics_tag(value: PairMinimumSemantics) -> u8 {
    match value {
        PairMinimumSemantics::ObservedDirectedPairs => 1,
    }
}

pub(super) fn minimum_cardinality_enforcement_tag(value: MinimumCardinalityEnforcement) -> u8 {
    match value {
        MinimumCardinalityEnforcement::CommitBoundary => 1,
        MinimumCardinalityEnforcement::CertificationBoundary => 2,
    }
}

pub(super) fn uniqueness_scope_tag(value: UniquenessScope) -> u8 {
    match value {
        UniquenessScope::DirectedSemanticEdge => 1,
        UniquenessScope::NormalizedSymmetricEdge => 2,
    }
}

pub(super) fn symmetry_mode_tag(value: SymmetryMode) -> u8 {
    match value {
        SymmetryMode::CanonicalUndirected => 1,
        SymmetryMode::PairedInverseRequired => 2,
        SymmetryMode::InverseProhibited => 3,
        SymmetryMode::PairedTwinRequired => 4,
    }
}

pub(super) fn endpoint_deletion_integrity_mode_tag(value: EndpointDeletionIntegrityMode) -> u8 {
    match value {
        EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations => 1,
        EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit => 2,
        EndpointDeletionIntegrityMode::RequireRelationRetirement => 3,
    }
}

pub(super) fn directed_traversal_kind_tag(value: DirectedTraversalKind) -> u8 {
    match value {
        DirectedTraversalKind::SourceToTarget => 1,
    }
}

pub(super) fn allowed_cycle_class_tag(value: AllowedCycleClass) -> u8 {
    match value {
        AllowedCycleClass::NoCycles => 1,
    }
}

pub(super) fn partition_isolation_mode_tag(value: PartitionIsolationMode) -> u8 {
    match value {
        PartitionIsolationMode::SamePartitionEndpoints => 1,
    }
}

pub(super) fn connectivity_minimum_enforcement_tag(value: ConnectivityMinimumEnforcement) -> u8 {
    match value {
        ConnectivityMinimumEnforcement::SnapshotPublication => 1,
    }
}
