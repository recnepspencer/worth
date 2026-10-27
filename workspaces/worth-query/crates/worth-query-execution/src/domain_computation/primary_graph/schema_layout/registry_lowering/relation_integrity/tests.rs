use worth_query_installation::facade::{
    ApplicationRelationCardinality, ApplicationRelationCrossContextPolicy,
    ApplicationRelationDeletionPolicy, ApplicationRelationEndpoints, ApplicationRelationIntegrity,
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
    let original =
        lower_relation_integrity("Peer", KindId(11), vec![KindId(11)], &baseline.into(), None)
            .unwrap();
    let isolated = lower_relation_integrity(
        "Peer",
        KindId(11),
        vec![KindId(11)],
        &baseline.same_partition_endpoints().into(),
        None,
    )
    .unwrap();
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
    let original =
        lower_relation_integrity("Peer", KindId(11), vec![KindId(11)], &baseline.into(), None)
            .unwrap();
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
            &baseline.symmetric().unique(declared).into(),
            None,
        )
        .unwrap();
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
        ApplicationRelationCardinality::new(Some(1), Some(2), Some(3), Some(4), Some(5), Some(6)),
        ApplicationRelationDeletionPolicy::RequireRelationDeletionInSameCommit,
    );

    let lowered = lower_relation_integrity(
        "Contains",
        KindId(11),
        vec![KindId(12)],
        &declared.into(),
        None,
    )
    .unwrap();

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
        &ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().into(),
        None,
    )
    .unwrap();

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
        &ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling()
            .into(),
        None,
    )
    .unwrap();

    assert_eq!(lowered.endpoint_kind_contracts.len(), 1);
    assert!(!lowered.endpoint_kind_contracts[0].self_edges_allowed);
    assert!(lowered.cardinality_contracts.is_empty());
    assert!(lowered.endpoint_deletion_integrity_contracts.is_empty());
}

#[test]
fn minimum_uses_the_complete_carried_kind_binding_and_native_id_order() {
    use std::collections::BTreeMap;
    let binding = BTreeMap::from([("A".into(), KindId(99)), ("Z".into(), KindId(4))]);
    let baseline = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    let lowered = lower_relation_integrity(
        "Reaches",
        KindId(11),
        vec![KindId(4)],
        &baseline.reaches_at_least(2, &["A", "Z", "A"]).into(),
        Some(&binding),
    )
    .unwrap();
    assert_eq!(lowered.connectivity_minimum_contracts.len(), 1);
    let minimum = &lowered.connectivity_minimum_contracts[0];
    assert_eq!(minimum.source_kind_ids, vec![KindId(11)]);
    assert_eq!(minimum.target_kind_ids, vec![KindId(4), KindId(99)]);
    assert_eq!(minimum.minimum_reachable_targets, 2);
    assert_eq!(
        minimum.enforcement_boundary,
        super::ConnectivityMinimumEnforcement::SnapshotPublication
    );
    assert_eq!(
        lower_relation_integrity(
            "Reaches",
            KindId(11),
            vec![KindId(4)],
            &baseline.into(),
            Some(&binding)
        )
        .unwrap(),
        lower_relation_integrity(
            "Reaches",
            KindId(11),
            vec![KindId(4)],
            &baseline.into(),
            None
        )
        .unwrap()
    );
}

#[test]
fn enabled_minimum_cannot_silently_lose_missing_kind_bindings() {
    let minimum = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling()
        .reaches_at_least(1, &["Unknown"]);
    assert!(lower_relation_integrity(
        "Reaches",
        KindId(11),
        vec![KindId(12)],
        &minimum.into(),
        None
    )
    .is_err());
    let binding = std::collections::BTreeMap::from([("Known".into(), KindId(12))]);
    assert!(lower_relation_integrity(
        "Reaches",
        KindId(11),
        vec![KindId(12)],
        &minimum.into(),
        Some(&binding)
    )
    .is_err());
}

#[test]
fn decoded_owned_minimum_names_use_the_same_native_lowerer() {
    use worth_query_installation::facade::{
        ApplicationRelationIntegrityRecord, ApplicationRelationMinimumReachabilityRecord,
    };
    let name = format!("DecodedTarget_{}", 42);
    let binding = std::collections::BTreeMap::from([(name.clone(), KindId(12))]);
    let mut record: ApplicationRelationIntegrityRecord =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().into();
    record.reaches_at_least = Some(ApplicationRelationMinimumReachabilityRecord {
        minimum_reachable_targets: 2,
        target_kinds: vec![name],
    });
    let lowered = lower_relation_integrity(
        "DecodedReaches",
        KindId(11),
        vec![KindId(12)],
        &record,
        Some(&binding),
    )
    .unwrap();
    let minimum = &lowered.connectivity_minimum_contracts[0];
    assert_eq!(minimum.source_kind_ids, vec![KindId(11)]);
    assert_eq!(minimum.target_kind_ids, vec![KindId(12)]);
    assert_eq!(minimum.minimum_reachable_targets, 2);
}

#[test]
fn orbit_adds_exact_native_cardinality_without_replacing_independent_policies() {
    let declared = ApplicationRelationIntegrity::new(
        ApplicationRelationEndpoints::same_context(false),
        ApplicationRelationCardinality::new(None, Some(2), None, None, None, Some(1)),
        ApplicationRelationDeletionPolicy::RequireRelationRetirement,
    );
    let lower = |value: ApplicationRelationIntegrity, targets| {
        lower_relation_integrity("Next", KindId(11), targets, &value.into(), None)
    };
    let original = lower(declared, vec![KindId(11)]).unwrap();
    let orbit = lower(declared.orbit(), vec![KindId(11)]).unwrap();
    assert_eq!(
        orbit.endpoint_kind_contracts,
        original.endpoint_kind_contracts
    );
    assert_eq!(
        orbit.endpoint_deletion_integrity_contracts,
        original.endpoint_deletion_integrity_contracts
    );
    assert_eq!(orbit.cardinality_contracts.len(), 2);
    assert_eq!(
        orbit.cardinality_contracts[0],
        original.cardinality_contracts[0]
    );
    let exact = &orbit.cardinality_contracts[1];
    assert_eq!(exact.contract_id.as_str(), "Next:orbit");
    assert_eq!((exact.source_min, exact.source_max), (Some(1), Some(1)));
    assert_eq!((exact.target_min, exact.target_max), (Some(1), Some(1)));
    assert_eq!((exact.pair_min, exact.pair_max), (None, None));
    assert_eq!(
        exact.minimum_enforcement,
        super::MinimumCardinalityEnforcement::CommitBoundary
    );
    assert_ne!(orbit.plan_revision, original.plan_revision);
    for targets in [vec![], vec![KindId(12)], vec![KindId(11), KindId(12)]] {
        assert!(lower(declared.orbit(), targets).is_err());
    }
}
