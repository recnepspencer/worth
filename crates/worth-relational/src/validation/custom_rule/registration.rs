use std::fmt;
use std::panic::{catch_unwind, AssertUnwindSafe, RefUnwindSafe, UnwindSafe};
use std::sync::Arc;

use crate::validation::engine::InvariantRuntimeView;

use super::execution_context::CustomInvariantExecutionContext;
use super::scope_planner::CustomInvariantScopePlanner;
use crate::validation::data::{
    CustomInvariantDescriptor, CustomInvariantRuleId, CustomInvariantSemanticIdentity,
    CustomInvariantVerdict,
};
use crate::validation::data::{
    CustomInvariantExecutionError, CustomInvariantFailure, CustomInvariantFailureKind,
    CustomInvariantPreparationError, CustomInvariantRuntimePhase,
    PreparedCustomInvariantExecutionOutcome,
};

pub trait CustomInvariantRule: Send + Sync + RefUnwindSafe + 'static {
    type Scope: Send + Sync + 'static;

    fn descriptor(&self) -> CustomInvariantDescriptor;

    fn prepare_scope(
        &self,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Result<Self::Scope, CustomInvariantPreparationError>;

    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        scope: &Self::Scope,
    ) -> Result<CustomInvariantVerdict, CustomInvariantExecutionError>;
}

pub(crate) trait PreparedCustomInvariantExecution: Send + Sync {
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
    ) -> PreparedCustomInvariantExecutionOutcome;

    fn work_meter(&self) -> super::CustomInvariantWorkMeter;
}

pub(crate) trait ErasedCustomInvariantRule: Send + Sync {
    fn prepare_for_execution(
        &self,
        runtime: &InvariantRuntimeView,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Arc<dyn PreparedCustomInvariantExecution>;
}

struct CustomInvariantAdapter<R: CustomInvariantRule> {
    rule: Arc<R>,
    identity: CustomInvariantSemanticIdentity,
}

struct PreparedCustomInvariantAdapter<R: CustomInvariantRule> {
    rule: Arc<R>,
    identity: CustomInvariantSemanticIdentity,
    scope: R::Scope,
    work: super::CustomInvariantWorkMeter,
}

struct FailedPreparedCustomInvariantExecution {
    failure: CustomInvariantFailure,
    work: super::CustomInvariantWorkMeter,
}

impl<R: CustomInvariantRule> PreparedCustomInvariantExecution
    for PreparedCustomInvariantAdapter<R>
{
    fn evaluate(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
    ) -> PreparedCustomInvariantExecutionOutcome {
        if self.work.exceeded() {
            return PreparedCustomInvariantExecutionOutcome::Failure(
                CustomInvariantFailure::execution_error(
                    &self.identity,
                    CustomInvariantExecutionError::new(
                        "custom invariant execution exhausted its installed work budget",
                    ),
                ),
            );
        }
        context
            .performance_access()
            .count_custom_invariant_execution();
        match run_custom_rule_safely(
            self.identity.clone(),
            CustomInvariantRuntimePhase::Execution,
            || self.rule.evaluate(context, &self.scope),
        ) {
            Ok(Ok(verdict)) => PreparedCustomInvariantExecutionOutcome::Verdict(verdict),
            Ok(Err(error)) => PreparedCustomInvariantExecutionOutcome::Failure(
                CustomInvariantFailure::execution_error(&self.identity, error),
            ),
            Err(failure) => PreparedCustomInvariantExecutionOutcome::Failure(failure),
        }
    }

    fn work_meter(&self) -> super::CustomInvariantWorkMeter {
        self.work.clone()
    }
}

impl PreparedCustomInvariantExecution for FailedPreparedCustomInvariantExecution {
    fn evaluate(
        &self,
        _context: &CustomInvariantExecutionContext<'_>,
    ) -> PreparedCustomInvariantExecutionOutcome {
        PreparedCustomInvariantExecutionOutcome::Failure(self.failure.clone())
    }

    fn work_meter(&self) -> super::CustomInvariantWorkMeter {
        self.work.clone()
    }
}

impl<R: CustomInvariantRule> ErasedCustomInvariantRule for CustomInvariantAdapter<R> {
    fn prepare_for_execution(
        &self,
        runtime: &InvariantRuntimeView,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Arc<dyn PreparedCustomInvariantExecution> {
        let identity = self.identity.clone();
        let work = planner.work_meter();
        if work.exceeded() {
            return Arc::new(FailedPreparedCustomInvariantExecution {
                failure: CustomInvariantFailure::preparation_error(
                    &identity,
                    CustomInvariantPreparationError::new(
                        "custom invariant scope exhausted its installed work budget",
                    ),
                ),
                work,
            });
        }
        runtime
            .performance_access()
            .count_custom_invariant_preparation();
        match run_custom_rule_safely(
            identity.clone(),
            CustomInvariantRuntimePhase::Preparation,
            || self.rule.prepare_scope(planner),
        ) {
            Ok(Ok(scope)) => Arc::new(PreparedCustomInvariantAdapter {
                rule: Arc::clone(&self.rule),
                identity,
                scope,
                work,
            }),
            Ok(Err(error)) => Arc::new(FailedPreparedCustomInvariantExecution {
                failure: CustomInvariantFailure::preparation_error(&identity, error),
                work,
            }),
            Err(failure) => {
                if failure.kind == CustomInvariantFailureKind::Panic {
                    runtime.performance_access().count_custom_invariant_panic();
                }
                Arc::new(FailedPreparedCustomInvariantExecution { failure, work })
            }
        }
    }
}

fn run_custom_rule_safely<T>(
    identity: CustomInvariantSemanticIdentity,
    phase: CustomInvariantRuntimePhase,
    run: impl FnOnce() -> T,
) -> Result<T, CustomInvariantFailure> {
    catch_unwind(AssertUnwindSafe(run)).map_err(|panic_value| {
        CustomInvariantFailure::panic(&identity, phase, panic_value_message(panic_value))
    })
}

fn panic_value_message(panic_value: Box<dyn std::any::Any + Send>) -> Arc<str> {
    if let Some(message) = panic_value.downcast_ref::<&'static str>() {
        return Arc::from(*message);
    }
    if let Some(message) = panic_value.downcast_ref::<String>() {
        return Arc::from(message.as_str());
    }
    Arc::from("custom invariant panicked with a non-string value")
}

#[derive(Clone)]
pub struct CustomInvariantRegistration {
    descriptor: CustomInvariantDescriptor,
    executable: Arc<dyn ErasedCustomInvariantRule>,
}

impl fmt::Debug for CustomInvariantRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CustomInvariantRegistration")
            .field("descriptor", &self.descriptor)
            .finish_non_exhaustive()
    }
}

impl CustomInvariantRegistration {
    pub fn new<R>(rule: R) -> Result<Self, CustomInvariantRegistrationError>
    where
        R: CustomInvariantRule + UnwindSafe,
    {
        let descriptor = rule.descriptor();
        Self::validate_descriptor(&descriptor)?;
        let executable = Arc::new(CustomInvariantAdapter {
            rule: Arc::new(rule),
            identity: descriptor.identity.clone(),
        });
        Ok(Self {
            descriptor,
            executable,
        })
    }

    pub fn descriptor(&self) -> &CustomInvariantDescriptor {
        &self.descriptor
    }

    pub fn execution_point(&self) -> crate::validation::data::InvariantExecutionPoint {
        self.descriptor.operational.execution_point
    }

    pub fn groups(&self) -> crate::validation::data::InvariantGroupSet {
        self.descriptor.operational.groups
    }

    pub fn cost_class(&self) -> crate::validation::data::InvariantCostClass {
        self.descriptor.operational.cost_class
    }

    pub fn failure_effect(&self) -> crate::validation::data::InvariantFailureEffect {
        self.descriptor.operational.failure_effect
    }

    pub fn maximum_work_units(&self) -> std::num::NonZeroU64 {
        self.descriptor.operational.maximum_work_units
    }

    pub fn access_contract(&self) -> &crate::validation::data::CustomInvariantAccessContract {
        &self.descriptor.operational.access
    }

    pub fn rule_id(&self) -> &CustomInvariantRuleId {
        &self.descriptor.identity.rule_id
    }

    pub(crate) fn executable(&self) -> &Arc<dyn ErasedCustomInvariantRule> {
        &self.executable
    }

    fn validate_descriptor(
        descriptor: &CustomInvariantDescriptor,
    ) -> Result<(), CustomInvariantRegistrationError> {
        if descriptor.identity.rule_id.as_str().trim().is_empty() {
            return Err(CustomInvariantRegistrationError::EmptyRuleId);
        }
        if descriptor.display_name.trim().is_empty() {
            return Err(CustomInvariantRegistrationError::EmptyDisplayName);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CustomInvariantRegistrationError {
    EmptyRuleId,
    EmptyDisplayName,
}
