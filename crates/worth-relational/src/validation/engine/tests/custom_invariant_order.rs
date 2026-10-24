//! The first violation in installation order wins at every worker count.
//! Rules are evaluated concurrently; the reducer orders their results by
//! installed packet index, never by completion order or rule identity.
use std::num::NonZeroUsize;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use worth_execution::{CancellationToken, LeaseRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

use super::custom_rules::AlwaysViolatesCustomRule;
use super::validation_engine_fixtures::*;
use crate::authority::commit::preparation::planning::strategy::PreparationStrategySelection;
use crate::facade::runtime::CustomInvariantLeaseBudget;
use crate::validation::data::{CustomInvariantRefusal, InvariantViolationFields};

/// Whether the later-installed rule has finished.
#[derive(Default)]
struct Finished {
    second: Mutex<bool>,
    signal: Condvar,
}

/// Installed first, its identity sorts last. It waits for the second rule to
/// finish before it violates, so under a fan-out it completes last.
struct First {
    finished: Arc<Finished>,
    wait: Duration,
    overlapped: Arc<Mutex<bool>>,
}

/// Installed second, its identity sorts first. It violates at once.
struct Second {
    finished: Arc<Finished>,
}

fn descriptor(rule_id: &str) -> CustomInvariantDescriptor {
    let mut descriptor = AlwaysViolatesCustomRule.descriptor();
    descriptor.identity.rule_id = CustomInvariantRuleId::new(rule_id);
    descriptor
}

fn refusal(ordinal: u8) -> CustomInvariantRefusal {
    CustomInvariantRefusal::DeclaredValue {
        value_identity: "test.custom.order.refusal".to_owned(),
        bytes: vec![ordinal],
    }
}

impl CustomInvariantRule for First {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        descriptor("test.custom.order.z-installed-first")
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
        _: &mut CustomInvariantScopePlanner<'_>,
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, CustomInvariantPreparationError>> {
        Some(Ok(()))
    }

    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        let finished = self.finished.second.lock().unwrap();
        let (finished, _) = self
            .finished
            .signal
            .wait_timeout_while(finished, self.wait, |second| !*second)
            .unwrap();
        *self.overlapped.lock().unwrap() = *finished;
        Ok(CustomInvariantVerdict::Violation(refusal(1)))
    }

    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        Some(self.evaluate(context, scope))
    }
}

impl CustomInvariantRule for Second {
    type Scope = ();

    fn descriptor(&self) -> CustomInvariantDescriptor {
        descriptor("test.custom.order.a-installed-second")
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
        _: &mut CustomInvariantScopePlanner<'_>,
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<Self::Scope, CustomInvariantPreparationError>> {
        Some(Ok(()))
    }

    fn evaluate(
        &self,
        _: &CustomInvariantExecutionContext<'_>,
        _: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError> {
        *self.finished.second.lock().unwrap() = true;
        self.finished.signal.notify_all();
        Ok(CustomInvariantVerdict::Violation(refusal(2)))
    }

    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
        _: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Result<CustomInvariantVerdict, CustomInvariantExecutionError>> {
        Some(self.evaluate(context, scope))
    }
}

struct Run {
    selected: Option<PreparationStrategySelection>,
    overlapped: bool,
    winner: (String, CustomInvariantRefusal),
}

fn run(workers: usize, wait: Duration) -> Run {
    let finished = Arc::new(Finished::default());
    let overlapped = Arc::new(Mutex::new(false));
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(RelationalSchemaRegistry::new())
        .custom_invariant(
            CustomInvariantRegistration::new(First {
                finished: Arc::clone(&finished),
                wait,
                overlapped: Arc::clone(&overlapped),
            })
            .unwrap(),
        )
        .custom_invariant(
            CustomInvariantRegistration::new(Second {
                finished: Arc::clone(&finished),
            })
            .unwrap(),
        )
        .build();
    let lease = crate::tests::support::test_execution_authority()
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    NonZeroUsize::new(workers).unwrap(),
                    Some(8 * 1024 * 1024),
                    Some(1_000_000),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .unwrap();
    let request = InvariantExecutionRequest::from_profile_with_contract(
        InvariantRequestProfile::CommitBoundary,
        &runtime,
        InvariantObservation::committed(runtime.storage_access().current_edition()),
        runtime.current_version_id(),
        None,
        None,
    );
    let result = InvariantEngine::new(&runtime)
        .execute(&worth_execution::ExecutionRequest::leased(&lease), request)
        .expect("both rules evaluate");
    let failure = result
        .summary()
        .blocking_failure()
        .expect("a violation blocks the commit");
    let InvariantViolationFields::CustomInvariantViolation { identity, refusal } = failure.fields()
    else {
        panic!("the winner is a custom refusal: {:?}", failure.fields());
    };
    let overlapped = *overlapped.lock().unwrap();
    Run {
        selected: result
            .metadata()
            .preparation_strategy()
            .map(|strategy| strategy.selected_mode),
        overlapped,
        winner: (identity.rule_id.as_str().to_owned(), refusal.clone()),
    }
}

#[test]
fn first_installed_violation_wins_at_one_two_and_eight_workers() {
    let first_installed = || ("test.custom.order.z-installed-first".to_owned(), refusal(1));
    assert_eq!(run(1, Duration::ZERO).winner, first_installed());
    for workers in [2, 8] {
        // The first rule returns only after the second has finished, so it
        // completes last; seeing the second finish shows both ran at once.
        let parallel = run(workers, Duration::from_secs(30));
        assert_eq!(
            parallel.selected,
            Some(PreparationStrategySelection::StagedParallel),
            "{workers} workers fan custom rules out"
        );
        assert!(
            parallel.overlapped,
            "{workers} workers ran both rules at once"
        );
        assert_eq!(parallel.winner, first_installed(), "{workers} workers");
    }
}
