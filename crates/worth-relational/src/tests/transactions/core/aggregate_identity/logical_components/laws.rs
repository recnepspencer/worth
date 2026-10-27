//! Canonical laws are enforced without aggregate, cache or custom-rule readers.
use super::fixtures::*;
use crate::identity::data::RelationId;
use crate::transactions::data::ConflictClass;
use crate::validation::data::InvariantViolationFields;
use std::collections::BTreeSet;

pub(super) fn laws() -> DerivedRelationGraphLaws {
    DerivedRelationGraphLaws {
        self_edges_allowed: true,
        cross_context_policy: CrossContextPolicy::Forbid,
        cardinality: DerivedRelationCardinality::default(),
        acyclic: false,
        symmetric: false,
        orbit: false,
        same_partition_endpoints: true,
        reaches_at_least: None,
    }
}
fn law_runtime(definition: DerivedRelationDefinitionParts) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(
            source_registry()
                .register_derived_relations(vec![definition])
                .unwrap(),
        )
        .build()
}
pub(super) fn try_commit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    batch: WorkerIntentBatch,
) -> Result<CommitResult, TransactionCommitError> {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime)
}
pub(super) fn current_truth(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> (
    BTreeSet<EntityId>,
    BTreeSet<(RelationId, KindId, EntityId, EntityId)>,
    Option<(
        crate::history::data::CommitId,
        crate::identity::data::VersionId,
    )>,
) {
    // Acquire the actual current head, rather than re-reading a retained old pin.
    let snapshot = snapshot_for_owner_branch(runtime, &BranchId("main".into()));
    let truth = runtime
        .read_truth()
        .read_snapshot(&snapshot, execution)
        .unwrap()
        .unwrap();
    let entities = truth.entities().iter().map(|e| e.entity_id).collect();
    let relations = truth
        .relations()
        .iter()
        .map(|r| (r.relation_id, r.kind.kind_id, r.source, r.target))
        .collect();
    drop(truth);
    runtime.snapshots().release_snapshot(&snapshot).unwrap();
    let head = runtime
        .history()
        .branch_head(&BranchId("main".into()))
        .map(|head| (head.commit_id, head.version_id));
    (entities, relations, head)
}
pub(super) fn violation(
    error: TransactionCommitError,
    law: DerivedRelationLaw,
) -> DerivedRelationLawViolation {
    let TransactionCommitError::Conflict { error, .. } = error else {
        panic!("canonical derived-law conflict required: {error:?}")
    };
    let ConflictClass::InvariantViolation {
        fields:
            InvariantViolationFields::DerivedRelationGraph {
                definition,
                violation,
            },
        ..
    } = error.class
    else {
        panic!("actual derived-law diagnostic required: {:?}", error.class)
    };
    assert_eq!(definition, identity("Law"));
    assert_eq!(violation.law, law);
    violation
}
fn incidence_law(orbit: bool) -> DerivedRelationDefinitionParts {
    use DerivedRelationNode::*;
    let mut graph_laws = laws();
    graph_laws.acyclic = !orbit;
    graph_laws.orbit = orbit;
    let mut nodes = vec![
        Stored {
            relation: KindId(INCIDENCE),
            from: KindId(1),
            to: KindId(6),
        },
        Reverse { input: 0 },
        Compose { left: 0, right: 1 },
    ];
    if !orbit {
        nodes.push(WithoutSelfEdges { input: 2 });
        graph_laws.self_edges_allowed = false;
    }
    DerivedRelationDefinitionParts {
        identity: identity("Law"),
        from: KindId(1).into(),
        to: KindId(1).into(),
        nodes,
        laws: Some(graph_laws),
    }
}
fn disjoint_birth() -> WorkerIntentBatch {
    let mut batch = WorkerIntentBatch::new("disjoint-source-birth");
    for index in 0..2 {
        batch = batch
            .push(entity(1, index))
            .push(entity(6, index))
            .push(edge(
                &format!("own-vertex-{index}"),
                INCIDENCE,
                EntityReference::Created(created(1, index)),
                EntityReference::Created(created(6, index)),
            ));
    }
    batch
}

#[test]
fn incidence_cycle_and_orbit_laws_refuse_without_aggregate_cache_or_rule_trigger() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    for (orbit, expected) in [
        (false, DerivedRelationLaw::Acyclic),
        (true, DerivedRelationLaw::Orbit),
    ] {
        let runtime = law_runtime(incidence_law(orbit));
        let born = commit(execution, &runtime, disjoint_birth());
        let carriers = (0..2)
            .map(|i| born.created_entity(&created(1, i)).unwrap())
            .collect::<Vec<_>>();
        let peer_vertex = born.created_entity(&created(6, 1)).unwrap();
        let before = current_truth(execution, &runtime);
        let failure = try_commit(
            execution,
            &runtime,
            WorkerIntentBatch::new("join-two-incidences").push(edge(
                "join-peer-vertex",
                INCIDENCE,
                EntityReference::Existing(carriers[0]),
                EntityReference::Existing(peer_vertex),
            )),
        )
        .unwrap_err();
        let failure = violation(failure, expected);
        assert!(carriers
            .iter()
            .any(|id| failure.member == NativeMemberKey::Entity(*id)));
        assert!(!failure.physical_relations.is_empty());
        assert!(failure.physical_entities.contains(&peer_vertex));
        assert_eq!(current_truth(execution, &runtime), before);
        // A genuinely separate vertex preserves both authored laws.
        commit(
            execution,
            &runtime,
            WorkerIntentBatch::new("lawful-disjoint-twin")
                .push(entity(1, 2))
                .push(entity(6, 2))
                .push(edge(
                    "separate-vertex",
                    INCIDENCE,
                    EntityReference::Created(created(1, 2)),
                    EntityReference::Created(created(6, 2)),
                )),
        );
        assert_eq!(
            current_truth(execution, &runtime).0.len(),
            before.0.len() + 2
        );
    }
}

fn lift_minimum(both_cases: bool) -> DerivedRelationDefinitionParts {
    use DerivedRelationNode::*;
    let mut nodes = vec![
        CarrierLift {
            domain: domain(),
            case: CASES[0].into(),
        },
        Reverse { input: 0 },
    ];
    if both_cases {
        nodes.extend([
            CarrierLift {
                domain: domain(),
                case: CASES[1].into(),
            },
            Reverse { input: 2 },
            Union { left: 1, right: 3 },
        ]);
    }
    let mut graph_laws = laws();
    graph_laws.cardinality.source_min = Some(1);
    DerivedRelationDefinitionParts {
        identity: identity("Law"),
        from: NativeMemberDomain::Projected {
            projected: domain(),
        },
        to: KindId(1).into(),
        nodes,
        laws: Some(graph_laws),
    }
}
#[test]
fn isolated_carrier_birth_checks_every_domain_case_without_physical_couplings() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let runtime = law_runtime(lift_minimum(false));
    let before = current_truth(execution, &runtime);
    let failure = try_commit(
        execution,
        &runtime,
        WorkerIntentBatch::new("isolated-birth").push(entity(1, 0)),
    )
    .unwrap_err();
    let failure = violation(failure, DerivedRelationLaw::SourceMinimum);
    let NativeMemberKey::Projected(member) = &failure.member else {
        panic!("actual projected case required")
    };
    assert_eq!(member.domain, domain());
    assert_eq!(member.case, CASES[1]);
    assert_eq!((failure.count, failure.limit), (0, 1));
    assert!(failure.physical_entities.contains(&member.carrier));
    assert!(failure.physical_relations.is_empty());
    assert_eq!(current_truth(execution, &runtime), before);
    let lawful = law_runtime(lift_minimum(true));
    let born = commit(
        execution,
        &lawful,
        WorkerIntentBatch::new("both-case-lift").push(entity(1, 0)),
    );
    assert_eq!(
        current_truth(execution, &lawful).0,
        BTreeSet::from([born.created_entity(&created(1, 0)).unwrap()])
    );
}
