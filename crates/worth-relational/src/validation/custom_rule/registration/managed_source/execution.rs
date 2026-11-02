//! Join every Native-discovered scope through the existing registered rule.
use crate::identity::data::EntityId;
use crate::validation::candidate_source::ManagedIndexSourceProgram;
use crate::validation::custom_rule::managed_source::ManagedSourceInvocation;
use crate::validation::custom_rule::registration::{
    ErasedCustomInvariantRule, FailedPreparedCustomInvariantExecution,
    PreparedCustomInvariantExecution,
};
use crate::validation::custom_rule::{
    CustomInvariantExecutionContext, CustomInvariantLeaseBudget, CustomInvariantScopePlanner,
    CustomInvariantWorkMeter,
};
use crate::validation::data::{
    CustomInvariantExecutionError, CustomInvariantFailure, CustomInvariantPreparationError,
    CustomInvariantSemanticIdentity, CustomInvariantVerdict,
    PreparedCustomInvariantExecutionOutcome as Outcome,
};
use crate::validation::engine::InvariantRuntimeView;
use std::sync::Arc;

pub(super) struct SourceScopedRule {
    inner: Arc<dyn ErasedCustomInvariantRule>,
    sources: Arc<[ManagedIndexSourceProgram]>,
    identity: CustomInvariantSemanticIdentity,
}

impl SourceScopedRule {
    pub(super) fn new(
        inner: Arc<dyn ErasedCustomInvariantRule>,
        sources: Arc<[ManagedIndexSourceProgram]>,
        identity: CustomInvariantSemanticIdentity,
    ) -> Self {
        Self {
            inner,
            sources,
            identity,
        }
    }

    fn scopes(
        &self,
        planner: &CustomInvariantScopePlanner<'_>,
    ) -> Result<Vec<EntityId>, Arc<dyn PreparedCustomInvariantExecution>> {
        crate::validation::candidate_source::observed_scopes(
            &self.sources,
            &planner.committed_relations(),
            &planner.relations(),
        )
        .map_err(|cause| {
            self.failed(
                planner,
                format!("managed Source invocation refused: {cause:?}"),
            )
        })
    }

    fn failed(
        &self,
        planner: &CustomInvariantScopePlanner<'_>,
        detail: impl Into<Arc<str>>,
    ) -> Arc<dyn PreparedCustomInvariantExecution> {
        Arc::new(FailedPreparedCustomInvariantExecution {
            failure: CustomInvariantFailure::preparation_error(
                &self.identity,
                CustomInvariantPreparationError::new(detail),
            ),
            work: planner.work_meter(),
        })
    }

    fn prepare(
        &self,
        runtime: &InvariantRuntimeView,
        planner: &mut CustomInvariantScopePlanner<'_>,
        checked: Option<&dyn CustomInvariantLeaseBudget>,
    ) -> Option<Arc<dyn PreparedCustomInvariantExecution>> {
        let scopes = match self.scopes(planner) {
            Ok(scopes) => scopes,
            Err(failed) => return Some(failed),
        };
        let work = planner.work_meter();
        let Some(bytes) = scopes
            .len()
            .checked_mul(size_of::<(
                ManagedSourceInvocation,
                Arc<dyn PreparedCustomInvariantExecution>,
            )>())
            .and_then(|n| n.checked_add(size_of::<ScopedExecutions>()))
            .and_then(|n| u64::try_from(n).ok())
        else {
            work.mark_exceeded();
            return Some(self.failed(planner, "managed Source invocation storage overflow"));
        };
        if !work.claim_result(bytes) {
            return Some(self.failed(planner, "managed Source invocation allocation refused"));
        }
        let mut children = Vec::new();
        if children.try_reserve_exact(scopes.len()).is_err() {
            work.mark_exceeded();
            return Some(self.failed(planner, "managed Source invocation allocation failed"));
        }
        for scope in scopes {
            if planner.check_live().is_err() || !work.checkpoint(1) {
                return Some(self.failed(planner, "managed Source invocation stopped"));
            }
            // Restored on every ordinary return. A panicking rule is caught by
            // its existing adapter; this owner never opens a replacement request.
            let invocation = ManagedSourceInvocation {
                scope,
                sources: self.sources.clone(),
                audience: self.identity.clone(),
            };
            let previous = planner.managed_scope.replace(invocation.clone());
            let child = match checked {
                Some(budget) => self
                    .inner
                    .prepare_for_execution_checked(runtime, planner, budget),
                None => Some(self.inner.prepare_for_execution(runtime, planner)),
            };
            planner.managed_scope = previous;
            children.push((invocation, child?));
        }
        Some(Arc::new(ScopedExecutions {
            children,
            work,
            identity: self.identity.clone(),
            checked_support: self.inner.supports_checked_preparation(),
        }))
    }
}

impl ErasedCustomInvariantRule for SourceScopedRule {
    fn supports_checked_preparation(&self) -> bool {
        self.inner.supports_checked_preparation()
    }

    fn prepare_for_execution(
        &self,
        runtime: &InvariantRuntimeView,
        planner: &mut CustomInvariantScopePlanner<'_>,
    ) -> Arc<dyn PreparedCustomInvariantExecution> {
        self.prepare(runtime, planner, None)
            .expect("ordinary preparation always returns its result")
    }

    fn prepare_for_execution_checked(
        &self,
        runtime: &InvariantRuntimeView,
        planner: &mut CustomInvariantScopePlanner<'_>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Arc<dyn PreparedCustomInvariantExecution>> {
        self.prepare(runtime, planner, Some(budget))
    }
}

struct ScopedExecutions {
    children: Vec<(
        ManagedSourceInvocation,
        Arc<dyn PreparedCustomInvariantExecution>,
    )>,
    work: CustomInvariantWorkMeter,
    identity: CustomInvariantSemanticIdentity,
    checked_support: bool,
}

impl ScopedExecutions {
    fn failure(&self, detail: &'static str) -> Outcome {
        Outcome::Failure(CustomInvariantFailure::execution_error(
            &self.identity,
            CustomInvariantExecutionError::new(detail),
        ))
    }
}

impl PreparedCustomInvariantExecution for ScopedExecutions {
    fn evaluate(&self, context: &CustomInvariantExecutionContext<'_>) -> Outcome {
        for (invocation, child) in &self.children {
            if context.check_live().is_err() {
                return self.failure("original managed Source request stopped");
            }
            match context
                .in_managed_source_invocation(invocation.clone(), |context| child.evaluate(context))
            {
                Outcome::Verdict(CustomInvariantVerdict::Pass) => {}
                refused => return refused,
            }
        }
        Outcome::Verdict(CustomInvariantVerdict::Pass)
    }

    fn evaluate_checked(
        &self,
        context: &CustomInvariantExecutionContext<'_>,
        budget: &dyn CustomInvariantLeaseBudget,
    ) -> Option<Outcome> {
        // A serial legacy rule may use the existing ordinary entrance. Refuse
        // before executing any child, so fallback cannot replay a partial group.
        if !self.checked_support {
            return None;
        }
        for (invocation, child) in &self.children {
            if context.check_live().is_err() {
                return Some(self.failure("original managed Source request stopped"));
            }
            match context.in_managed_source_invocation(invocation.clone(), |context| {
                child.evaluate_checked(context, budget)
            }) {
                Some(Outcome::Verdict(CustomInvariantVerdict::Pass)) => {}
                Some(refused) => return Some(refused),
                None => {
                    return Some(self.failure(
                        "declared checked managed Source rule returned no checked result",
                    ))
                }
            }
        }
        Some(Outcome::Verdict(CustomInvariantVerdict::Pass))
    }

    fn work_meter(&self) -> CustomInvariantWorkMeter {
        self.work.clone()
    }
}
