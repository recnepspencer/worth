//! Metadata-dependent laws see the coherent actual consequence adjacency.
use super::fixtures::*;
use super::laws::{current_truth, laws, try_commit, violation};

pub(super) fn law_registry(maximum: bool) -> RelationalSchemaRegistry {
    let mut graph_laws = laws();
    if maximum {
        graph_laws.cardinality.source_max = Some(1);
    } else {
        graph_laws.cardinality.source_min = Some(1);
    }
    let declaration = DerivedRelationDefinitionParts {
        identity: identity("Law"),
        from: KindId(1).into(),
        to: descriptor().storage.aggregate_kind.into(),
        nodes: vec![DerivedRelationNode::Stored {
            relation: descriptor().membership_kind,
            from: KindId(1),
            to: descriptor().storage.aggregate_kind,
        }],
        laws: Some(graph_laws),
    };
    registry()
        .register_derived_relations(vec![declaration])
        .unwrap()
}
fn runtime(maximum: bool) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(law_registry(maximum))
        .build()
        .expect("valid native runtime configuration")
}

#[test]
fn membership_laws_validate_after_real_case_memberships_exist_and_before_publication() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let lawful = runtime(false);
    let born = commit(
        execution,
        &lawful,
        WorkerIntentBatch::new("membership-minimum").push(entity(1, 0)),
    );
    let carrier = born.created_entity(&created(1, 0)).unwrap();
    let truth = current_truth(execution, &lawful);
    assert_eq!(
        truth
            .1
            .iter()
            .filter(|(_, kind, from, _)| *kind == descriptor().membership_kind && *from == carrier)
            .count(),
        2
    );
    assert_ne!(
        read(execution, &lawful, &born.snapshot, key(carrier, 0)).identity(),
        read(execution, &lawful, &born.snapshot, key(carrier, 1)).identity(),
    );
    // The same final two edges violate a real maximum. An early business-only
    // view would see zero and incorrectly admit this transaction.
    let limited = runtime(true);
    let before = current_truth(execution, &limited);
    let error = try_commit(
        execution,
        &limited,
        WorkerIntentBatch::new("membership-maximum").push(entity(1, 0)),
    )
    .unwrap_err();
    let failure = violation(error, DerivedRelationLaw::SourceMaximum);
    assert!(matches!(failure.member, NativeMemberKey::Entity(_)));
    assert_eq!((failure.count, failure.limit), (2, 1));
    assert_eq!(failure.physical_relations.len(), 2);
    assert_eq!(current_truth(execution, &limited), before);
}
