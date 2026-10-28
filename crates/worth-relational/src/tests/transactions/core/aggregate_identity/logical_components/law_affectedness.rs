//! Reverse and transitive failures are scheduled by actual source edits alone.
use super::fixtures::*;
use super::laws::{current_truth, laws, try_commit, violation};
const SUCCESSOR: u32 = 13;

fn runtime(minimum: bool) -> RelationalRuntime {
    let source = RelationKindRegistration {
        kind_id: KindId(SUCCESSOR),
        kind_name: "direct-law-source".into(),
        schema_id: domain().schema,
        schema_version_id: SchemaVersionId(1),
        cross_context_policy: CrossContextPolicy::Forbid,
        cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
        aspect_contract_declarations: Default::default(),
        relation_integrity: RelationIntegrityDeclarations::new(
            vec![EndpointKindContractDeclaration {
                contract_id: "direct-law-source:endpoints".into(),
                allowed_source_kinds: vec![KindId(1)],
                allowed_target_kinds: vec![KindId(1)],
                self_edges_allowed: true,
                cross_context_policy: CrossContextPolicy::Forbid,
            }],
            vec![],
            vec![],
            vec![],
            vec![],
        ),
    };
    let mut graph_laws = laws();
    if minimum {
        graph_laws.reaches_at_least = Some(DerivedRelationMinimumReachability {
            minimum_reachable_targets: 2,
            target_domains: vec![KindId(1).into()],
        });
    } else {
        graph_laws.symmetric = true;
    }
    let declaration = DerivedRelationDefinitionParts {
        identity: identity("Law"),
        from: KindId(1).into(),
        to: KindId(1).into(),
        nodes: vec![DerivedRelationNode::Stored {
            relation: KindId(SUCCESSOR),
            from: KindId(1),
            to: KindId(1),
        }],
        laws: Some(graph_laws),
    };
    RelationalRuntimeApi::builder()
        .schema_registry(
            source_registry()
                .register_relation_kind(source)
                .unwrap()
                .register_derived_relations(vec![declaration])
                .unwrap(),
        )
        .build()
}
fn delete(relation: crate::identity::data::RelationId) -> MutationIntent {
    MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id: relation,
    }))
}
#[test]
fn deleting_only_a_reverse_source_refuses_and_deleting_the_whole_pair_is_lawful() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime(false);
    let born = commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("symmetric-pair")
            .push(entity(1, 0))
            .push(entity(1, 1))
            .push(edge(
                "forward",
                SUCCESSOR,
                EntityReference::Created(created(1, 0)),
                EntityReference::Created(created(1, 1)),
            ))
            .push(edge(
                "reverse",
                SUCCESSOR,
                EntityReference::Created(created(1, 1)),
                EntityReference::Created(created(1, 0)),
            )),
    );
    let a = born.created_entity(&created(1, 0)).unwrap();
    let b = born.created_entity(&created(1, 1)).unwrap();
    let before = current_truth(execution, &runtime);
    let forward = before
        .1
        .iter()
        .find(|(_, kind, from, to)| *kind == KindId(SUCCESSOR) && *from == a && *to == b)
        .unwrap()
        .0;
    let reverse = before
        .1
        .iter()
        .find(|(_, kind, from, to)| *kind == KindId(SUCCESSOR) && *from == b && *to == a)
        .unwrap()
        .0;
    let failure = violation(
        try_commit(
            execution,
            &runtime,
            WorkerIntentBatch::new("delete-reverse").push(delete(reverse)),
        )
        .unwrap_err(),
        DerivedRelationLaw::Symmetric,
    );
    assert_eq!(failure.member, a.into());
    assert_eq!(failure.counterpart, Some(b.into()));
    assert!(failure.physical_relations.contains(&forward));
    assert_eq!(current_truth(execution, &runtime), before);
    commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("delete-whole-pair")
            .push(delete(forward))
            .push(delete(reverse)),
    );
    assert!(current_truth(execution, &runtime).1.is_empty());
}

#[test]
fn downstream_source_loss_rechecks_an_unchanged_predecessors_transitive_minimum() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = runtime(true);
    let mut batch = WorkerIntentBatch::new("minimum-three-cycle");
    for index in 0..3 {
        batch = batch.push(entity(1, index)).push(edge(
            &format!("cycle-{index}"),
            SUCCESSOR,
            EntityReference::Created(created(1, index)),
            EntityReference::Created(created(1, (index + 1) % 3)),
        ));
    }
    let born = commit(execution, &runtime, batch);
    let ids = (0..3)
        .map(|i| born.created_entity(&created(1, i)).unwrap())
        .collect::<Vec<_>>();
    let index = (0..3).max_by_key(|i| ids[*i]).unwrap();
    let edited = ids[index];
    let predecessor = ids[(index + 2) % 3];
    let untouched = ids[(index + 1) % 3];
    assert!(predecessor < edited);
    let before = current_truth(execution, &runtime);
    let removed = before
        .1
        .iter()
        .find(|(_, kind, from, _)| *kind == KindId(SUCCESSOR) && *from == edited)
        .unwrap()
        .0;
    let preserved = before
        .1
        .iter()
        .find(|(_, kind, from, to)| {
            *kind == KindId(SUCCESSOR) && *from == predecessor && *to == edited
        })
        .unwrap()
        .0;
    // Literal remaining paths: predecessor→edited (one other member), while
    // untouched→predecessor→edited still reaches two. The greatest source is
    // chosen so its own zero-target failure cannot hide the predecessor check.
    assert_ne!(untouched, predecessor);
    let failure = violation(
        try_commit(
            execution,
            &runtime,
            WorkerIntentBatch::new("cut-downstream").push(delete(removed)),
        )
        .unwrap_err(),
        DerivedRelationLaw::MinimumReachability,
    );
    assert_eq!(failure.member, predecessor.into());
    assert_eq!((failure.count, failure.limit), (1, 2));
    assert!(failure.physical_relations.contains(&preserved));
    assert_eq!(current_truth(execution, &runtime), before);
}
