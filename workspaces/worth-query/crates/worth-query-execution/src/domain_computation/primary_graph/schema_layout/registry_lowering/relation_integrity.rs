use super::lower_cross_context;
use worth_query_installation::facade::{
    ApplicationRelationDeletionPolicy, ApplicationRelationIntegrity,
    ApplicationRelationUniquenessScope,
};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::schema::{
    AcyclicityContractDeclaration, AllowedCycleClass, CardinalityContractDeclaration, ContractId,
    DirectedTraversalKind, EndpointDeletionIntegrityDeclaration, EndpointDeletionIntegrityMode,
    EndpointKindContractDeclaration, MinimumCardinalityEnforcement, PairMinimumSemantics,
    PartitionIsolationContractDeclaration, PartitionIsolationMode, RelationIntegrityDeclarations,
    SymmetryContractDeclaration, SymmetryMode, UniquenessContractDeclaration, UniquenessScope,
};

pub(super) fn lower_relation_integrity(
    relation: &str,
    from_kind: KindId,
    to_kinds: Vec<KindId>,
    integrity: ApplicationRelationIntegrity,
) -> RelationIntegrityDeclarations {
    let endpoints = vec![EndpointKindContractDeclaration {
        contract_id: ContractId::new(format!("{relation}:endpoints")),
        allowed_source_kinds: vec![from_kind],
        allowed_target_kinds: to_kinds,
        self_edges_allowed: integrity.endpoints.self_edges_allowed,
        cross_context_policy: lower_cross_context(integrity.endpoints.cross_context_policy),
    }];
    let cardinality = integrity
        .cardinality
        .has_bound()
        .then(|| CardinalityContractDeclaration {
            contract_id: ContractId::new(format!("{relation}:cardinality")),
            source_max: integrity.cardinality.source_max,
            target_max: integrity.cardinality.target_max,
            pair_max: integrity.cardinality.pair_max,
            source_min: integrity.cardinality.source_min,
            target_min: integrity.cardinality.target_min,
            pair_min: integrity.cardinality.pair_min,
            pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
            minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
        });
    let deletion = lower_endpoint_deletion(relation, integrity.deletion);
    let acyclicity = integrity.acyclic.then(|| AcyclicityContractDeclaration {
        contract_id: ContractId::new(format!("{relation}:acyclic")),
        traversal_direction: DirectedTraversalKind::SourceToTarget,
        allowed_cycle_class: AllowedCycleClass::NoCycles,
    });
    RelationIntegrityDeclarations::new(
        endpoints,
        cardinality.into_iter().collect(),
        integrity
            .unique
            .map(|scope| UniquenessContractDeclaration {
                contract_id: ContractId::new(format!("{relation}:unique")),
                scope: match scope {
                    ApplicationRelationUniquenessScope::DirectedSemanticEdge => {
                        UniquenessScope::DirectedSemanticEdge
                    }
                    ApplicationRelationUniquenessScope::NormalizedSymmetricEdge => {
                        UniquenessScope::NormalizedSymmetricEdge
                    }
                },
            })
            .into_iter()
            .collect(),
        integrity
            .symmetric
            .then(|| SymmetryContractDeclaration {
                contract_id: ContractId::new(format!("{relation}:symmetry")),
                mode: SymmetryMode::PairedTwinRequired,
            })
            .into_iter()
            .collect(),
        deletion.into_iter().collect(),
    )
    .with_acyclicity_contracts(acyclicity.into_iter().collect())
    .with_partition_isolation_contracts(
        integrity
            .same_partition_endpoints
            .then(|| PartitionIsolationContractDeclaration {
                contract_id: ContractId::new(format!("{relation}:same_partition_endpoints")),
                isolation_mode: PartitionIsolationMode::SamePartitionEndpoints,
            })
            .into_iter()
            .collect(),
    )
}

fn lower_endpoint_deletion(
    relation: &str,
    value: ApplicationRelationDeletionPolicy,
) -> Option<EndpointDeletionIntegrityDeclaration> {
    let mode = match value {
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations => {
            EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations
        }
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit => {
            EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit
        }
        ApplicationRelationDeletionPolicy::RequireRelationRetirement => {
            EndpointDeletionIntegrityMode::RequireRelationRetirement
        }
        ApplicationRelationDeletionPolicy::RetainDanglingForAudit
        | ApplicationRelationDeletionPolicy::CascadeDeleteRelations => return None,
    };
    Some(EndpointDeletionIntegrityDeclaration {
        contract_id: ContractId::new(format!("{relation}:deletion")),
        mode,
    })
}

#[cfg(test)]
mod tests {
    use worth_query_installation::facade::{
        ApplicationRelationCardinality, ApplicationRelationCrossContextPolicy,
        ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints,
        ApplicationRelationIntegrity,
    };
    use worth_relational::facade::identity::KindId;
    use worth_relational::facade::schema::EndpointDeletionIntegrityMode;

    use super::lower_relation_integrity;

    #[test]
    fn declared_partition_isolation_is_lowered_independently_of_endpoint_permission() {
        let baseline = ApplicationRelationIntegrity::new(
            ApplicationRelationEndpoints::new(
                true,
                ApplicationRelationCrossContextPolicy::AllowExplicit,
            ),
            ApplicationRelationCardinality::unbounded(),
            ApplicationRelationDeletionPolicy::RetainDanglingForAudit,
        );
        let original = lower_relation_integrity("Peer", KindId(11), vec![KindId(11)], baseline);
        let isolated = lower_relation_integrity(
            "Peer",
            KindId(11),
            vec![KindId(11)],
            baseline.same_partition_endpoints(),
        );
        assert!(original.partition_isolation_contracts.is_empty());
        assert_eq!(isolated.partition_isolation_contracts.len(), 1);
        assert_eq!(
            isolated.partition_isolation_contracts[0].isolation_mode,
            super::PartitionIsolationMode::SamePartitionEndpoints
        );
        assert_eq!(
            isolated.endpoint_kind_contracts,
            original.endpoint_kind_contracts
        );
        assert_ne!(isolated.plan_revision, original.plan_revision);
    }

    #[test]
    fn paired_symmetry_and_exact_uniqueness_scope_lower_into_one_native_plan() {
        use worth_query_installation::facade::ApplicationRelationUniquenessScope as Declared;
        use worth_relational::facade::schema::{SymmetryMode, UniquenessScope as Native};
        let baseline = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
        let original = lower_relation_integrity("Peer", KindId(11), vec![KindId(11)], baseline);
        let mut revisions = Vec::new();
        for (declared, native) in [
            (Declared::DirectedSemanticEdge, Native::DirectedSemanticEdge),
            (
                Declared::NormalizedSymmetricEdge,
                Native::NormalizedSymmetricEdge,
            ),
        ] {
            let lowered = lower_relation_integrity(
                "Peer",
                KindId(11),
                vec![KindId(11)],
                baseline.symmetric().unique(declared),
            );
            assert_eq!(lowered.symmetry_contracts.len(), 1);
            assert_eq!(
                lowered.symmetry_contracts[0].mode,
                SymmetryMode::PairedTwinRequired
            );
            assert_eq!(lowered.uniqueness_contracts.len(), 1);
            assert_eq!(lowered.uniqueness_contracts[0].scope, native);
            assert_ne!(lowered.plan_revision, original.plan_revision);
            revisions.push(lowered.plan_revision);
        }
        assert_ne!(revisions[0], revisions[1]);
    }

    #[test]
    fn declared_relation_integrity_lowers_exact_endpoint_cardinality_and_deletion_contracts() {
        let declared = ApplicationRelationIntegrity::new(
            ApplicationRelationEndpoints::new(
                true,
                ApplicationRelationCrossContextPolicy::SchemaControlled,
            ),
            ApplicationRelationCardinality::new(
                Some(1),
                Some(2),
                Some(3),
                Some(4),
                Some(5),
                Some(6),
            ),
            ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
        );

        let lowered = lower_relation_integrity("Contains", KindId(11), vec![KindId(12)], declared);

        let endpoints = &lowered.endpoint_kind_contracts[0];
        assert_eq!(endpoints.allowed_source_kinds, vec![KindId(11)]);
        assert_eq!(endpoints.allowed_target_kinds, vec![KindId(12)]);
        assert!(endpoints.self_edges_allowed);
        let cardinality = &lowered.cardinality_contracts[0];
        assert_eq!(
            (cardinality.source_min, cardinality.source_max),
            (Some(1), Some(2))
        );
        assert_eq!(
            (cardinality.target_min, cardinality.target_max),
            (Some(3), Some(4))
        );
        assert_eq!(
            (cardinality.pair_min, cardinality.pair_max),
            (Some(5), Some(6))
        );
        assert_eq!(
            lowered.endpoint_deletion_integrity_contracts[0].mode,
            EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit,
        );
    }

    #[test]
    fn unbounded_relation_omits_a_false_cardinality_contract() {
        let lowered = lower_relation_integrity(
            "Contains",
            KindId(11),
            vec![KindId(12)],
            ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
        );

        assert_eq!(lowered.endpoint_kind_contracts.len(), 1);
        assert!(lowered.endpoint_kind_contracts[0].self_edges_allowed);
        assert!(lowered.cardinality_contracts.is_empty());
        assert!(lowered.endpoint_deletion_integrity_contracts.is_empty());
    }

    #[test]
    fn no_self_edges_uses_the_same_relation_integrity_lowering_path() {
        let lowered = lower_relation_integrity(
            "Contains",
            KindId(11),
            vec![KindId(11)],
            ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling(),
        );

        assert_eq!(lowered.endpoint_kind_contracts.len(), 1);
        assert!(!lowered.endpoint_kind_contracts[0].self_edges_allowed);
        assert!(lowered.cardinality_contracts.is_empty());
        assert!(lowered.endpoint_deletion_integrity_contracts.is_empty());
    }
}
