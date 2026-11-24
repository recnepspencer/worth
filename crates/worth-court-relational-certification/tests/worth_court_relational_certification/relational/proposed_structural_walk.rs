//! Ordinary retargeting must supply the actual proposed and before-image views.
use super::pinned_read_publication::snapshot_for_main;
use super::proposal_records::{create_entity_with_fields, create_relation_with_fields};
use std::sync::{Arc, Mutex, OnceLock};
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_foundational::facade::RelationDirection;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{
        CompleteStructuralWalk, CustomInvariantAccessContract, CustomInvariantDescriptor,
        CustomInvariantExecutionContext, CustomInvariantExecutionError,
        CustomInvariantOperationalMetadata, CustomInvariantPreparationError,
        CustomInvariantRegistration, CustomInvariantRule, CustomInvariantRuleId,
        CustomInvariantScopePlanner, CustomInvariantSemanticIdentity,
        CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
        InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
        PinnedStructuralWalkScope, RelationalRuntimeApi, StructuralRelationView,
    },
    transactions::{
        EntityReference, MutationIntent, RelationMutationIntent, UpdateRelationEndpointsIntent,
        WorkerIntentBatch,
    },
};
use worth_testkit_relational::{
    aspect_storage::{
        publications::{begin_main_transaction, release_test_commit_snapshot},
        AspectSchemaFixture,
    },
    shared_relational_certification_test_execution_policy::serial_request,
};

fn visits(walk: &CompleteStructuralWalk) -> Vec<(EntityId, usize)> {
    walk.steps()
        .iter()
        .map(|step| {
            assert_eq!(step.kind, KindId(1));
            (step.entity, step.distance)
        })
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ObservedViews {
    evaluation: bool,
    proposed: Vec<(EntityId, usize)>,
    committed: Vec<(EntityId, usize)>,
}

struct CandidateWalkRule {
    root: Arc<OnceLock<EntityId>>,
    observations: Arc<Mutex<Vec<ObservedViews>>>,
}

impl CandidateWalkRule {
    fn observe(
        &self,
        proposed: StructuralRelationView<'_>,
        committed: StructuralRelationView<'_>,
        evaluation: bool,
    ) {
        let Some(root) = self.root.get() else {
            return;
        };
        let proposed = proposed.walk_outgoing_from(&[*root], KindId(2)).unwrap();
        let committed = committed.walk_outgoing_from(&[*root], KindId(2)).unwrap();
        self.observations.lock().unwrap().push(ObservedViews {
            evaluation,
            proposed: visits(&proposed),
            committed: visits(&committed),
        });
    }
}

impl CustomInvariantRule for CandidateWalkRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new("test.complete-candidate-walk"),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Complete candidate walk"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1)],
                    read_relation_kinds: vec![KindId(2)],
                    affected_entity_kinds: vec![KindId(1)],
                    affected_relation_kinds: vec![KindId(2)],
                    include_relation_endpoint_entity_touches: true,
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        self.observe(planner.relations(), planner.committed_relations(), false);
        Ok(())
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        self.observe(context.relations(), context.committed_relations(), true);
        Ok(CustomInvariantVerdict::Pass)
    }
}

#[test]
fn ordinary_retarget_walks_actual_proposed_and_committed_views() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let root = Arc::new(OnceLock::new());
    let observations = Arc::new(Mutex::new(Vec::new()));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(AspectSchemaFixture::default().build_registry())
        .custom_invariant(
            CustomInvariantRegistration::new(CandidateWalkRule {
                root: Arc::clone(&root),
                observations: Arc::clone(&observations),
            })
            .unwrap(),
        )
        .build()
        .unwrap();
    let entities = ["root", "source", "old-target", "new-target"]
        .map(|name| create_entity_with_fields(&execution, &runtime, name, Default::default()));
    create_relation_with_fields(
        &execution,
        &runtime,
        entities[0],
        entities[1],
        "first",
        Default::default(),
    );
    let edge = create_relation_with_fields(
        &execution,
        &runtime,
        entities[1],
        entities[2],
        "retargeted",
        Default::default(),
    );
    root.set(entities[0]).unwrap();
    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("retarget").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: edge,
                kind_id: KindId(2),
                source: EntityReference::Existing(entities[1]),
                target: EntityReference::Existing(entities[3]),
            }),
        )),
        ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    let outcome = txn.commit(&execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &outcome);
    let expected_proposed = vec![(entities[0], 0), (entities[1], 1), (entities[3], 2)];
    let expected_committed = vec![(entities[0], 0), (entities[1], 1), (entities[2], 2)];
    assert_eq!(
        *observations.lock().unwrap(),
        [
            ObservedViews {
                evaluation: false,
                proposed: expected_proposed.clone(),
                committed: expected_committed.clone()
            },
            ObservedViews {
                evaluation: true,
                proposed: expected_proposed.clone(),
                committed: expected_committed
            },
        ]
    );
    let pinned = runtime
        .read_truth()
        .pin_read_capability(&snapshot_for_main(&runtime), &[], &execution)
        .unwrap();
    let current = pinned
        .complete_relation_walk(
            &execution,
            PinnedStructuralWalkScope::new(
                &entities[..1],
                KindId(1),
                KindId(2),
                RelationDirection::Forward,
            ),
        )
        .unwrap();
    let visits = current
        .visits()
        .iter()
        .map(|visit| {
            assert_eq!(visit.entity.kind_id, KindId(1));
            (visit.entity.entity_id, visit.distance)
        })
        .collect::<Vec<_>>();
    assert_eq!(visits, expected_proposed);
}
