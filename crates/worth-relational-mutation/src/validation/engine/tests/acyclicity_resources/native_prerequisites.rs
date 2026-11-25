//! Native law refusal precedes every application scope hook on the candidate.
pub mod effects;
#[cfg(test)]
pub mod interruptions;
#[cfg(test)]
pub mod preparation_stops;
#[cfg(test)]
pub mod retained_evidence;

use super::super::custom_rules::AlwaysViolatesCustomRule;
use super::super::validation_engine_fixtures::*;
#[cfg(test)]
use super::packet_capacity::{paid_dag_request, InvariantDag};
#[cfg(test)]
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::validation::custom_rule::CustomInvariantLeaseBudget;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
#[cfg(test)]
use worth_execution::CancellationToken;
use worth_execution::ExecutionRequest;
#[cfg(test)]
use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};

#[derive(Default)]
pub struct Hooks {
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
    pub fn observed(&self) -> (usize, usize) {
        (
            self.prepared.load(Ordering::SeqCst),
            self.evaluated.load(Ordering::SeqCst),
        )
    }
}

pub struct ScopeProbe(pub Arc<Hooks>);

#[cfg(test)]
fn fixture(count: usize) -> (InvariantDag, Arc<Hooks>) {
    let hooks = Arc::new(Hooks::default());
    let rule = CustomInvariantRegistration::new(ScopeProbe(Arc::clone(&hooks))).unwrap();
    (InvariantDag::new(count, None, Some(rule)), hooks)
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
