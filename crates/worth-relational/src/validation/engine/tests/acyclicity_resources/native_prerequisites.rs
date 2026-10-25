//! Native law refusal precedes every application scope hook on the candidate.
mod effects;
mod interruptions;
mod preparation_stops;
mod retained_evidence;
mod work_account;

use super::super::custom_rules::AlwaysViolatesCustomRule;
use super::super::validation_engine_fixtures::*;
use super::fixture::DagFixture;
use crate::facade::runtime::CustomInvariantLeaseBudget;
use crate::transactions::data::{
    ConflictClass, EntityReference, RecordRef, TransactionCommitError,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_execution::{CancellationToken, ExecutionRequest, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

#[derive(Default)]
struct Hooks {
    prepared: AtomicUsize,
    evaluated: AtomicUsize,
    preparation: std::sync::Mutex<Preparation>,
    native_probe: std::sync::Mutex<Option<Arc<worth_execution::SerialRequest>>>,
    native_accounting: std::sync::Mutex<Vec<bool>>,
}

#[derive(Default)]
enum Preparation {
    #[default]
    Pass,
    Refuse,
    Cancel(worth_execution::CancellationSource),
}

impl Hooks {
    fn observed(&self) -> (usize, usize) {
        (
            self.prepared.load(Ordering::SeqCst),
            self.evaluated.load(Ordering::SeqCst),
        )
    }
}

struct ScopeProbe(Arc<Hooks>);

fn fixture(count: usize) -> (DagFixture, Arc<Hooks>) {
    let hooks = Arc::new(Hooks::default());
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(acyclicity_and_connectivity_registry())
        .custom_invariant(CustomInvariantRegistration::new(ScopeProbe(Arc::clone(&hooks))).unwrap())
        .build();
    (DagFixture::from_runtime(runtime, count), hooks)
}

impl CustomInvariantRule for ScopeProbe {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        let mut descriptor = AlwaysViolatesCustomRule.descriptor();
        descriptor.identity.rule_id = CustomInvariantRuleId::new("test.native-prerequisite.scope");
        descriptor.operational.maximum_work_units = None;
        descriptor.operational.access.read_entity_kinds = vec![KindId(3)];
        descriptor.operational.access.affected_entity_kinds.clear();
        descriptor
    }

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<(), CustomInvariantPreparationError> {
        planner
            .check_live()
            .map_err(|stop| CustomInvariantPreparationError::new(format!("{stop:?}")))?;
        self.0.prepared.fetch_add(1, Ordering::SeqCst);
        if let Some(backing) = self.0.native_probe.lock().unwrap().as_ref() {
            ExecutionRequest::serial(backing)
                .run_native_with_checkpoint(|context| {
                    self.0
                        .native_accounting
                        .lock()
                        .unwrap()
                        .push(context.has_accounting());
                })
                .map_err(|stop| CustomInvariantPreparationError::new(format!("{stop:?}")))?;
        }
        match &*self.0.preparation.lock().unwrap() {
            Preparation::Pass => Ok(()),
            Preparation::Refuse => Err(CustomInvariantPreparationError::new(
                "probe preparation refusal",
            )),
            Preparation::Cancel(source) => {
                source.cancel();
                planner
                    .check_live()
                    .map_err(|stop| CustomInvariantPreparationError::new(format!("{stop:?}")))
            }
        }
    }

    fn supports_checked_preparation(&self) -> bool {
        true
    }

    fn prepare_scope_checked(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<(), CustomInvariantPreparationError>> {
        Some(self.prepare_scope(planner))
    }

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &(),
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        context
            .check_live()
            .map_err(|stop| CustomInvariantExecutionError::new(format!("{stop:?}")))?;
        self.0.evaluated.fetch_add(1, Ordering::SeqCst);
        Ok(CustomInvariantVerdict::Pass)
    }

    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &(),
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        Some(self.evaluate(context, scope))
    }
}

#[test]
fn a_native_cycle_refuses_before_custom_preparation_and_its_one_edge_twin_commits() {
    for posture in [
        None,
        Some(ExecutionPosture::Serial),
        Some(ExecutionPosture::Automatic),
    ] {
        let hooks = Arc::new(Hooks::default());
        let runtime = RelationalRuntimeApi::builder()
            .schema_registry(acyclicity_and_connectivity_registry())
            .custom_invariant(
                CustomInvariantRegistration::new(ScopeProbe(Arc::clone(&hooks))).unwrap(),
            )
            .build();
        let fixture = DagFixture::from_runtime(runtime, 2);
        let source = fixture.nodes[0];
        let target = fixture.nodes[1];
        let lease = crate::tests::support::test_execution_authority()
            .request_lease(LeaseRequest {
                policy: ExecutionRequestPolicy::new(
                    posture.unwrap_or(ExecutionPosture::Serial),
                    DeterminismContract::CanonicalBitwise,
                    ExecutionBudget::new(
                        std::num::NonZeroUsize::new(4).unwrap(),
                        Some(8 * 1024 * 1024),
                        Some(4_000_000),
                    ),
                ),
                cancellation: CancellationToken::new(),
                deadline: None,
            })
            .unwrap();
        let serial = worth_execution::SerialRequest::from_policy(
            lease.policy(),
            CancellationToken::new(),
            None,
        );
        let execution = if posture.is_some() {
            ExecutionRequest::leased(&lease)
        } else {
            ExecutionRequest::serial(&serial)
        };
        let charged_memory = || match posture {
            Some(_) => lease.charged_memory_bytes(),
            None => serial
                .memory()
                .expect("this finite serial request owns its memory budget")
                .charged_memory_bytes(),
        };
        let seeded = fixture
            .publish(
                &execution,
                fixture.edge_batch(target, source, "existing-reverse"),
            )
            .unwrap();
        let reverse = seeded
            .changed_records
            .iter()
            .filter_map(|record| match record {
                RecordRef::Relation(id) => Some(*id),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(reverse.len(), 1);
        crate::tests::support::release_test_commit_snapshot(&fixture.runtime, &seeded);
        let before = fixture.head();
        let baseline = hooks.observed();
        assert_eq!(
            baseline,
            (1, 1),
            "the same installed hook handles the lawful seed"
        );

        let failure = fixture
            .validate(&execution, fixture.edge_batch(source, target, "closing"))
            .expect_err("the single proposed edge closes the committed reverse edge");
        let TransactionCommitError::Conflict { error, .. } = failure else {
            panic!("original native conflict required: {failure:?}");
        };
        let ConflictClass::InvariantViolation {
            fields:
                InvariantViolationFields::Acyclicity {
                    contract_id,
                    relation_kind_id,
                    source: refused_source,
                    target: refused_target,
                },
            ..
        } = error.class
        else {
            panic!("original acyclicity fields required: {error:?}");
        };
        assert_eq!(contract_id.as_str(), "no_cycles");
        assert_eq!(relation_kind_id, KindId(2));
        assert_eq!(refused_source, EntityReference::Existing(source));
        assert_eq!(refused_target, EntityReference::Existing(target));
        assert_eq!(
            hooks.observed(),
            baseline,
            "native denial must protect custom preparation, {posture:?}"
        );
        assert_eq!(fixture.head(), before);
        assert_eq!(charged_memory(), 0);

        let twin = fixture
            .edge_batch(source, target, "closing")
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent {
                    relation_id: reverse[0],
                },
            )));
        let committed = fixture
            .publish(&execution, twin)
            .expect("removing one edge makes the same proposal acyclic");
        assert_ne!(fixture.head(), before);
        assert_eq!(hooks.observed(), (baseline.0 + 1, baseline.1 + 1));
        let boundary = committed
            .invariant_executions()
            .iter()
            .filter(|result| {
                result.metadata().execution_point()
                    == crate::validation::data::InvariantExecutionPoint::CommitBoundary
            })
            .collect::<Vec<_>>();
        assert_eq!(boundary.len(), 1, "both stages retain one boundary result");
        let result = boundary[0];
        assert_eq!(result.results().len(), 2);
        assert!(matches!(
            &result.results()[0].rule,
            InvariantReportedRule::Native(InvariantRule::AcyclicityContract(_))
        ));
        assert!(
            matches!(&result.results()[1].rule, InvariantReportedRule::Custom(identity)
            if identity.rule_id.as_str() == "test.native-prerequisite.scope")
        );
        assert_eq!(result.decision_log().len(), 2);
        assert_eq!(
            result.metadata().proof_boundary().unwrap().packet_count(),
            2
        );
        assert_eq!(
            result
                .metadata()
                .proof_boundary()
                .unwrap()
                .touched_partition_count(),
            1
        );
        crate::tests::support::release_test_commit_snapshot(&fixture.runtime, &committed);
        assert_eq!(charged_memory(), 0);
    }
}
