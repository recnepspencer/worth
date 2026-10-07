use std::num::{NonZeroU64, NonZeroUsize};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, OnceLock,
};

use worth_execution::{CancellationSource, CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

use super::custom_rules::AlwaysViolatesCustomRule;
use super::validation_engine_fixtures::*;
use crate::facade::runtime::CustomInvariantLeaseBudget;

fn checked_lease(
    memory_bytes: u64,
    work_ceiling: u64,
    cancellation: CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(NonZeroUsize::new(4).unwrap(), memory_bytes, work_ceiling),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}

struct CheckedCounterRule {
    steps: usize,
    visits: Arc<AtomicUsize>,
    preparation_visits: Arc<AtomicUsize>,
    cancellation: Option<CancellationSource>,
    preparation_steps: usize,
    preparation_scratch_bytes: u64,
    target: Option<Arc<OnceLock<crate::identity::data::EntityId>>>,
}

impl CustomInvariantRule for CheckedCounterRule {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        let mut descriptor = AlwaysViolatesCustomRule.descriptor();
        descriptor.identity.rule_id = CustomInvariantRuleId::new("test.custom.checked-counter");
        descriptor.operational.maximum_work_units = NonZeroU64::new(100_000).unwrap();
        descriptor
    }

    fn prepare_scope(
        &self,
        _: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError> {
        Ok(())
    }

    fn supports_checked_preparation(&self) -> bool {
        true
    }

    fn prepare_scope_checked(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, CustomInvariantPreparationError>> {
        if let Some(target) = self.target.as_ref().and_then(|target| target.get()) {
            if let Err(error) = planner.relations().entity_kind(*target) {
                return Some(Err(CustomInvariantPreparationError::new(format!(
                    "checked structural read: {error:?}"
                ))));
            }
        }
        for _ in 0..self.preparation_steps {
            if !budget.checkpoint(1) {
                break;
            }
            let visits = self.preparation_visits.fetch_add(1, Ordering::SeqCst);
            if visits == 1 {
                if let Some(cancellation) = &self.cancellation {
                    cancellation.cancel();
                }
            }
        }
        let _ = budget.claim_scratch(self.preparation_scratch_bytes);
        Some(Ok(()))
    }

    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        Ok(CustomInvariantVerdict::Pass)
    }

    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        _: &Self::Scope,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        for _ in 0..self.steps {
            if let Some(target) = self.target.as_ref().and_then(|target| target.get()) {
                let _ = context.relations().entity_kind(*target);
            }
            if !budget.checkpoint(1) {
                break;
            }
            let visits = self.visits.fetch_add(1, Ordering::SeqCst);
            if visits == 1 && self.preparation_steps == 0 {
                if let Some(cancellation) = &self.cancellation {
                    cancellation.cancel();
                }
            }
        }
        // The worker must propagate a failed meter claim even when a custom
        // evaluator incorrectly returns a successful verdict afterward.
        let _ = budget.claim_result(64);
        Some(Ok(CustomInvariantVerdict::Pass))
    }
}

fn execute(
    steps: usize,
    work_ceiling: u64,
) -> (
    usize,
    Result<
        crate::validation::engine::InvariantExecutionResult,
        crate::transactions::data::TransactionCommitError,
    >,
) {
    let visits = Arc::new(AtomicUsize::new(0));
    let preparation_visits = Arc::new(AtomicUsize::new(0));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(
            CustomInvariantRegistration::new(CheckedCounterRule {
                steps,
                visits: Arc::clone(&visits),
                preparation_visits,
                cancellation: None,
                preparation_steps: 3,
                preparation_scratch_bytes: 64,
                target: None,
            })
            .unwrap(),
        )
        .build();
    let lease = checked_lease(8 * 1024 * 1024, work_ceiling, CancellationToken::new());
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let result = InvariantEngine::new(&runtime).execute_with_lease(request, &lease);
    (visits.load(Ordering::SeqCst), result)
}

#[test]
fn checked_custom_rule_executes_under_lease() {
    let (visits, result) = execute(3, 1_000);
    assert_eq!(visits, 3);
    let result = result.expect("checked custom evaluator should run");
    assert!(matches!(
        result.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Pass
    ));
}

#[test]
fn checked_custom_rule_stops_mid_scan_when_work_is_exhausted() {
    let (visits, result) = execute(1_000, 100);
    assert!(visits > 0 && visits < 1_000, "visited {visits}");
    assert!(matches!(
        result,
        Err(
            crate::transactions::data::TransactionCommitError::Execution {
                denial: crate::transactions::data::CommitExecutionDenial {
                    kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::WorkExhausted
                    ),
                    ..
                },
                ..
            }
        )
    ));
}

#[test]
fn checked_custom_preparation_reads_declared_structural_state() {
    let target = Arc::new(OnceLock::new());
    let preparation_visits = Arc::new(AtomicUsize::new(0));
    let evaluation_visits = Arc::new(AtomicUsize::new(0));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(crate::tests::support::test_schema_registry())
        .custom_invariant(
            CustomInvariantRegistration::new(CheckedCounterRule {
                steps: 1,
                visits: Arc::clone(&evaluation_visits),
                preparation_visits: Arc::clone(&preparation_visits),
                cancellation: None,
                preparation_steps: 2,
                preparation_scratch_bytes: 64,
                target: Some(Arc::clone(&target)),
            })
            .unwrap(),
        )
        .build();
    let entity_id = create_entity_of_kind(&runtime, KindId(1), "checked-structural");
    target.set(entity_id).unwrap();
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let result = InvariantEngine::new(&runtime)
        .execute_with_lease(
            request,
            &checked_lease(8 * 1024 * 1024, 1_000_000, CancellationToken::new()),
        )
        .expect("checked structural preparation and evaluation succeed");
    assert_eq!(preparation_visits.load(Ordering::SeqCst), 2);
    assert_eq!(evaluation_visits.load(Ordering::SeqCst), 1);
    assert!(matches!(
        result.results()[0].verdict,
        crate::validation::data::InvariantVerdict::Pass
    ));
}

#[test]
fn checked_custom_preparation_observes_cancellation_mid_scan() {
    let cancellation = CancellationSource::new();
    let preparation_visits = Arc::new(AtomicUsize::new(0));
    let evaluation_visits = Arc::new(AtomicUsize::new(0));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(
            CustomInvariantRegistration::new(CheckedCounterRule {
                steps: 1,
                visits: Arc::clone(&evaluation_visits),
                preparation_visits: Arc::clone(&preparation_visits),
                cancellation: Some(cancellation.clone()),
                preparation_steps: 1_000,
                preparation_scratch_bytes: 64,
                target: None,
            })
            .unwrap(),
        )
        .build();
    let lease = checked_lease(8 * 1024 * 1024, 10_000, cancellation.token());
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let result = InvariantEngine::new(&runtime).execute_with_lease(request, &lease);
    assert_eq!(preparation_visits.load(Ordering::SeqCst), 2);
    assert_eq!(evaluation_visits.load(Ordering::SeqCst), 0);
    assert!(matches!(
        result,
        Err(
            crate::transactions::data::TransactionCommitError::Execution {
                denial: crate::transactions::data::CommitExecutionDenial {
                    kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::Cancelled
                    ),
                    ..
                },
                ..
            }
        )
    ));
}

#[test]
fn checked_custom_preparation_denies_memory_before_large_scratch_claim() {
    let evaluation_visits = Arc::new(AtomicUsize::new(0));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(
            CustomInvariantRegistration::new(CheckedCounterRule {
                steps: 1,
                visits: Arc::clone(&evaluation_visits),
                preparation_visits: Arc::new(AtomicUsize::new(0)),
                cancellation: None,
                preparation_steps: 1,
                preparation_scratch_bytes: 20_000,
                target: None,
            })
            .unwrap(),
        )
        .build();
    let lease = checked_lease(128 * 1024, 10_000, CancellationToken::new());
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let result = InvariantEngine::new(&runtime).execute_with_lease(request, &lease);
    assert_eq!(evaluation_visits.load(Ordering::SeqCst), 0);
    assert!(matches!(
        result,
        Err(
            crate::transactions::data::TransactionCommitError::Execution {
                denial: crate::transactions::data::CommitExecutionDenial {
                    kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::ScratchCapacityExceeded
                    ),
                    ..
                },
                ..
            }
        )
    ));
}

#[test]
fn checked_custom_evaluation_observes_cancellation_during_structural_views() {
    let cancellation = CancellationSource::new();
    let target = Arc::new(OnceLock::new());
    let visits = Arc::new(AtomicUsize::new(0));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(crate::tests::support::test_schema_registry())
        .custom_invariant(
            CustomInvariantRegistration::new(CheckedCounterRule {
                steps: 1_000,
                visits: Arc::clone(&visits),
                preparation_visits: Arc::new(AtomicUsize::new(0)),
                cancellation: Some(cancellation.clone()),
                preparation_steps: 0,
                preparation_scratch_bytes: 0,
                target: Some(Arc::clone(&target)),
            })
            .unwrap(),
        )
        .build();
    target
        .set(create_entity_of_kind(&runtime, KindId(1), "checked-cancel"))
        .unwrap();
    let lease = checked_lease(8 * 1024 * 1024, 10_000, cancellation.token());
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let result = InvariantEngine::new(&runtime).execute_with_lease(request, &lease);
    assert_eq!(visits.load(Ordering::SeqCst), 2);
    assert!(matches!(
        result,
        Err(
            crate::transactions::data::TransactionCommitError::Execution {
                denial: crate::transactions::data::CommitExecutionDenial {
                    kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::Cancelled
                    ),
                    ..
                },
                ..
            }
        )
    ));
}
