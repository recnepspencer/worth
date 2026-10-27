//! Real checked workers consume the same logical proposal and committed sources.
use super::super::read_handoff_rule::parallel::ParallelReads;
use super::{
    fixtures::*,
    lifecycle,
    oracle::{Anchor, Carrier, Oracle},
    seeded,
};
use crate::aggregate_identity::AggregateRead;
use crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection;
use crate::validation::data::*;
use std::{
    collections::BTreeSet,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};
use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
struct Control {
    root: Mutex<Option<EntityId>>,
}
struct Reader {
    id: usize,
    source: DerivedRelationHandle,
    case: ProjectedMemberCaseHandle,
    control: Arc<Control>,
    overlap: Arc<ParallelReads>,
}
struct Reading {
    proof: PreparedDerivedRelationRead,
    aggregate: AggregateRead<NativeMemberKey>,
}
struct Scope {
    proposed: Reading,
    committed: Reading,
}
impl Reader {
    fn read(&self, view: &StructuralRelationView<'_>, root: EntityId) -> Reading {
        let member = view
            .projected_member_from_carrier(&self.case, root)
            .unwrap();
        let proof = view
            .prepare_derived_source(
                &self.source,
                key(root, 0),
                DerivedRelationReadShape::Component,
            )
            .unwrap();
        let aggregate = view
            .aggregates()
            .of_projected_definition(descriptor().definition, &member)
            .unwrap();
        let data = view
            .read_prepared_derived_source(&proof, DerivedRelationReadShape::Component)
            .unwrap();
        assert_eq!(
            data.endpoints(),
            aggregate.members().cloned().collect::<Vec<_>>()
        );
        Reading { proof, aggregate }
    }
    fn verify(&self, view: &StructuralRelationView<'_>, reading: &Reading) {
        let root = self.control.root.lock().unwrap().unwrap();
        let data = view
            .read_prepared_derived_source(&reading.proof, DerivedRelationReadShape::Component)
            .unwrap();
        let member = view
            .projected_member_from_carrier(&self.case, root)
            .unwrap();
        let actual = view
            .aggregates()
            .of_projected_definition(descriptor().definition, &member)
            .unwrap();
        assert_eq!(
            actual.members().cloned().collect::<Vec<_>>(),
            data.endpoints()
        );
        assert_eq!(actual.identity(), reading.aggregate.identity());
        assert_eq!(
            actual.members().cloned().collect::<Vec<_>>(),
            reading.aggregate.members().cloned().collect::<Vec<_>>()
        );
    }
}
impl CustomInvariantRule for Reader {
    type Scope = Option<Scope>;
    fn descriptor(&self) -> CustomInvariantDescriptor {
        CustomInvariantDescriptor {
            identity: CustomInvariantSemanticIdentity {
                rule_id: CustomInvariantRuleId::new(format!("logical-worker-{}", self.id)),
                semantic_version: CustomInvariantSemanticVersion::new(1, 0),
            },
            display_name: Arc::from("Actual logical checked worker"),
            operational: CustomInvariantOperationalMetadata {
                maximum_work_units: None,
                access: CustomInvariantAccessContract {
                    read_entity_kinds: vec![KindId(1), KindId(6), KindId(7), KindId(8)],
                    read_relation_kinds: vec![
                        KindId(INCIDENCE),
                        KindId(NEG_REGION),
                        KindId(POS_REGION),
                        KindId(NEG_EXTERIOR),
                        KindId(POS_EXTERIOR),
                    ],
                    affected_entity_kinds: vec![KindId(1)],
                    affected_relation_kinds: vec![
                        KindId(INCIDENCE),
                        KindId(NEG_REGION),
                        KindId(POS_REGION),
                        KindId(NEG_EXTERIOR),
                        KindId(POS_EXTERIOR),
                    ],
                    include_relation_endpoint_entity_touches: true,
                },
                execution_point: InvariantExecutionPoint::CommitBoundary,
                groups: InvariantGroupSet::of(InvariantGroup::SchemaCompliance),
                cost_class: InvariantCostClass::Touched,
                failure_effect: InvariantFailureEffect::BlockCommit,
            },
        }
    }
    fn supports_checked_preparation(&self) -> bool {
        true
    }
    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        let Some(root) = *self.control.root.lock().unwrap() else {
            return Ok(None);
        };
        Ok(Some(Scope {
            proposed: self.read(&planner.relations(), root),
            committed: self.read(&planner.committed_relations(), root),
        }))
    }
    fn prepare_scope_checked(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, CustomInvariantPreparationError>> {
        if !budget.checkpoint(1) || !budget.claim_result(size_of::<Scope>() as u64) {
            return Some(Err(CustomInvariantPreparationError::new(
                "original logical scope stopped",
            )));
        }
        Some(self.prepare_scope(planner))
    }
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        if let Some(scope) = scope {
            self.overlap.enter(self.id);
            self.verify(&context.relations(), &scope.proposed);
            self.verify(&context.committed_relations(), &scope.committed);
            self.overlap.finish(self.id);
        }
        Ok(CustomInvariantVerdict::Pass)
    }
    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        if !budget.checkpoint(1) {
            return Some(Err(CustomInvariantExecutionError::new(
                "original logical evaluation stopped",
            )));
        }
        Some(self.evaluate(context, scope))
    }
}
#[test]
fn real_one_two_and_eight_workers_publish_identical_logical_cohorts_names_and_lineage() {
    let serial = run(1);
    for workers in [2, 8] {
        assert_eq!(
            run(workers),
            serial,
            "actual checked {workers}-worker execution"
        );
    }
}
fn run(workers: usize) -> Vec<Vec<seeded::Group>> {
    let schema = registry();
    let control = Arc::new(Control {
        root: Mutex::new(None),
    });
    let overlap = ParallelReads::new(workers);
    let source = schema.derived_relation(&identity("Adjacency")).unwrap();
    let case = schema
        .projected_member_domain(&domain())
        .unwrap()
        .case(CASES[0])
        .unwrap();
    let mut builder = RelationalRuntimeApi::builder().schema_registry(schema);
    for id in [10, 11] {
        builder = builder.custom_invariant(
            CustomInvariantRegistration::new(Reader {
                id,
                source: source.clone(),
                case: case.clone(),
                control: Arc::clone(&control),
                overlap: Arc::clone(&overlap),
            })
            .unwrap(),
        );
    }
    let runtime = builder.build();
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    NonZeroUsize::new(workers).unwrap(),
                    Some(128 * 1024 * 1024),
                    Some(10_000_000),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let execution = &ExecutionRequest::leased(&lease);
    let mut batch = WorkerIntentBatch::new("logical-workers-birth");
    for index in 0..4 {
        batch = batch.push(entity(1, index));
    }
    for index in 0..2 {
        batch = batch
            .push(entity(6, index))
            .push(entity(7, index))
            .push(entity(8, index));
    }
    for index in 0..4 {
        batch = batch.push(edge(
            &format!("worker-vertex-{index}"),
            INCIDENCE,
            EntityReference::Created(created(1, index)),
            EntityReference::Created(created(6, index / 2)),
        ));
        for side in 0..2 {
            batch = batch.push(edge(
                &format!("worker-anchor-{index}-{side}"),
                lifecycle::anchor_kind(side, Anchor::Exterior(0)),
                EntityReference::Created(created(1, index)),
                lifecycle::anchor_reference(Anchor::Exterior(0)),
            ));
        }
    }
    let born = commit(execution, &runtime, batch);
    let ids = (0..4)
        .map(|index| born.created_entity(&created(1, index)).unwrap())
        .collect::<Vec<_>>();
    let mut oracle = Oracle::default();
    for (index, id) in ids.iter().enumerate() {
        oracle.carriers.insert(
            *id,
            Carrier {
                vertices: BTreeSet::from([index / 2]),
                anchors: [Anchor::Exterior(0); 2],
            },
        );
    }
    oracle.check(execution, &runtime, &born);
    let mut states = vec![seeded::state(
        execution,
        &runtime,
        &born.snapshot,
        &oracle.groups(),
    )];
    *control.root.lock().unwrap() = Some(ids[0]);
    let mut current = born.snapshot.clone();
    for step in 0..3 {
        let mut batch = WorkerIntentBatch::new(format!("logical-workers-step-{step}"));
        let changes = match step {
            0 => vec![(ids[0], 1, Anchor::Region(0))],
            1 => vec![(ids[0], 1, Anchor::Exterior(0))],
            _ => ids[..2]
                .iter()
                .flat_map(|id| (0..2).map(move |side| (*id, side, Anchor::Region(1))))
                .collect(),
        };
        for (carrier, side, after) in changes {
            let before = oracle.carriers[&carrier].anchors[side];
            batch.intents.extend(
                lifecycle::replace_anchor(
                    execution,
                    &runtime,
                    &current,
                    carrier,
                    side,
                    before,
                    after,
                    &born,
                    &format!("worker-delta-{step}-{}-{side}", carrier.local_slot_value()),
                )
                .intents,
            );
            oracle.carriers.get_mut(&carrier).unwrap().anchors[side] = after;
        }
        overlap.reset();
        let result = commit(execution, &runtime, batch);
        if workers > 1 {
            assert!(
                result
                    .validation()
                    .invariant_executions()
                    .iter()
                    .any(|record| record
                        .metadata()
                        .preparation_strategy()
                        .is_some_and(|strategy| strategy.selected_mode
                            == PreparationStrategySelection::StagedParallel)),
                "real parallel mode"
            );
        }
        overlap.assert_overlap();
        oracle.check(execution, &runtime, &result);
        states.push(seeded::state(
            execution,
            &runtime,
            &result.snapshot,
            &oracle.groups(),
        ));
        current = result.snapshot;
    }
    states
}
