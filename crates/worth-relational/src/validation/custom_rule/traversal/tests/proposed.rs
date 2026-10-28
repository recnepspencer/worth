//! The ordinary native commit supplies real candidate and before-image views.
use std::sync::{Mutex, OnceLock};

use super::*;
use crate::facade::transactions::{RelationMutationIntent, UpdateRelationEndpointsIntent};
use crate::validation::custom_rule::{
    CustomInvariantExecutionContext, CustomInvariantRegistration, CustomInvariantRule,
    CustomInvariantScopePlanner,
};
use crate::validation::data::{
    CustomInvariantDescriptor, CustomInvariantExecutionError, CustomInvariantOperationalMetadata,
    CustomInvariantPreparationError, CustomInvariantRuleId, CustomInvariantSemanticIdentity,
    CustomInvariantSemanticVersion, CustomInvariantVerdict, InvariantCostClass,
    InvariantExecutionPoint, InvariantFailureEffect, InvariantGroup, InvariantGroupSet,
};

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
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let root = Arc::new(OnceLock::new());
    let observations = Arc::new(Mutex::new(Vec::new()));
    let graph = Graph::with_rules(
        &execution,
        4,
        &[(0, 1, KindId(2)), (1, 2, KindId(2))],
        vec![CustomInvariantRegistration::new(CandidateWalkRule {
            root: Arc::clone(&root),
            observations: Arc::clone(&observations),
        })
        .unwrap()],
    );
    root.set(graph.entities[0]).unwrap();
    let edge = graph
        .read(&execution, unbounded(), |view| {
            view.walk_outgoing_from(&[graph.entities[1]], KindId(2))
        })
        .unwrap()
        .relations()[0];
    let mut txn = test_owner_begin_transaction_for_main(&graph.runtime);
    txn.push_batch(
        WorkerIntentBatch::new("retarget").push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: edge.relation_id,
                kind_id: KindId(2),
                source: EntityReference::Existing(graph.entities[1]),
                target: EntityReference::Existing(graph.entities[3]),
            }),
        )),
        ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    let outcome = txn.commit(&execution, &graph.runtime).unwrap();
    release_test_commit_snapshot(&graph.runtime, &outcome);
    let expected_proposed = vec![
        (graph.entities[0], 0),
        (graph.entities[1], 1),
        (graph.entities[3], 2),
    ];
    let expected_committed = vec![
        (graph.entities[0], 0),
        (graph.entities[1], 1),
        (graph.entities[2], 2),
    ];
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
    let current = graph
        .read(&execution, unbounded(), |view| {
            view.walk_outgoing_from(&[graph.entities[0]], KindId(2))
        })
        .unwrap();
    assert_eq!(visits(&current), expected_proposed);
}
