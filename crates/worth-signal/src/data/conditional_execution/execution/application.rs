use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::output::NodeEvaluationResult;
use crate::logic::evaluation::{EvaluationVerdict, EvaluationWork};
use crate::logic::prepared::PreparedEvaluation;

use super::super::condition_resolution::{resolve_condition, ConditionDisposition};
use super::super::{
    InstalledSignalConditionResolver, SignalConditionalDecisionClass,
    SignalConditionalDecisionCounters,
};
use super::{ConditionalExecutionProviders, SignalConditionalExecutionRequest};

pub(super) struct ConditionalResolution {
    pub(super) class: SignalConditionalDecisionClass,
    pub(super) computed_evaluation: Option<crate::logic::evaluation::AppliedEffectReport>,
}

impl ConditionalResolution {
    pub(super) fn passive(class: SignalConditionalDecisionClass) -> Self {
        Self {
            class,
            computed_evaluation: None,
        }
    }
}

pub(super) struct ConditionalResolutionAttempt<
    'graph,
    'request,
    'contract,
    'provider_borrow,
    'providers,
    'counter,
    'observer,
    Condition,
    Comparator,
    Compute,
> {
    pub(super) graph: &'graph mut SignalGraph,
    pub(super) request: &'request SignalConditionalExecutionRequest<'contract>,
    pub(super) providers: &'provider_borrow mut ConditionalExecutionProviders<
        'providers,
        Condition,
        Comparator,
        Compute,
    >,
    pub(super) ready_invalidation:
        Option<crate::data::proof::invalidation::progression::ReadyInvalidationBatch>,
    pub(super) counters: &'counter mut SignalConditionalDecisionCounters,
    pub(super) work:
        &'counter mut crate::data::retained_storage::RetainedStoragePreparation<'observer>,
}

impl<Condition, Comparator, Compute>
    ConditionalResolutionAttempt<'_, '_, '_, '_, '_, '_, '_, Condition, Comparator, Compute>
where
    Condition: InstalledSignalConditionResolver,
    Comparator: ComparatorPolicyResolver,
    Compute: FnOnce() -> Result<NodeEvaluationResult, SignalError>,
{
    pub(super) fn resolve(self) -> Result<ConditionalResolution, SignalError> {
        super::work::reserve(self.work, Some(1))?;
        let disposition = resolve_condition(
            self.graph,
            self.request,
            self.providers.condition,
            self.work,
        )?;
        match disposition {
            ConditionDisposition::Eligible => self.apply_computed(),
            ConditionDisposition::Suppressed => {
                self.counters.application_contacts += 1;
                apply_passive(
                    self.graph,
                    self.request.contract.node(),
                    self.providers.comparator,
                    self.work,
                )?;
                Ok(ConditionalResolution::passive(
                    SignalConditionalDecisionClass::SuppressedBeforeCompute,
                ))
            }
            ConditionDisposition::Deferred => {
                self.apply_deferred(SignalConditionalDecisionClass::DeferredByCondition)
            }
            ConditionDisposition::DeferredTemporal => {
                self.apply_deferred(SignalConditionalDecisionClass::DeferredTemporal)
            }
            ConditionDisposition::DeferredOnDemand => {
                self.apply_deferred(SignalConditionalDecisionClass::DeferredOnDemand)
            }
        }
    }

    fn apply_computed(self) -> Result<ConditionalResolution, SignalError> {
        let compute = self.providers.compute.take().ok_or_else(|| {
            SignalError::internal("conditional compute provider was already consumed")
        })?;
        let compute = || {
            self.counters.compute_contacts += 1;
            compute()
        };
        let result = match self.ready_invalidation {
            Some(ready) => crate::logic::invalidation::scheduling::execute_ready_with_work(
                &*self.graph,
                ready,
                &mut EvaluationWork::Conditional(self.work),
                compute,
            )?,
            None => compute()?,
        };
        let prepared = PreparedEvaluation::from_result(result);
        self.counters.application_contacts += 1;
        let applied =
            crate::logic::evaluation::apply_prepared_evaluation_after_dependencies_with_policy(
                self.graph,
                self.request.contract.node(),
                prepared,
                self.providers.comparator,
                None,
                0,
                None,
                false,
                &mut EvaluationWork::Conditional(self.work),
            )?;
        self.counters.semantic_classifications += 1;
        let class = match &applied.report.verdict {
            EvaluationVerdict::Recomputed => {
                self.counters.semantic_changes += 1;
                SignalConditionalDecisionClass::ComputedChanged
            }
            EvaluationVerdict::Suppressed { .. } => {
                SignalConditionalDecisionClass::ComputedRevertedClean
            }
            EvaluationVerdict::Deferred { .. } => {
                return Err(SignalError::internal(
                    "computed conditional output cannot become a deferred verdict",
                ))
            }
        };
        Ok(ConditionalResolution {
            class,
            computed_evaluation: Some(applied.report),
        })
    }

    fn apply_deferred(
        self,
        class: SignalConditionalDecisionClass,
    ) -> Result<ConditionalResolution, SignalError> {
        self.counters.application_contacts += 1;
        apply_deferred(
            self.graph,
            self.request.contract.node(),
            self.providers.comparator,
            self.work,
        )?;
        Ok(ConditionalResolution::passive(class))
    }
}

pub(super) fn apply_passive(
    graph: &mut SignalGraph,
    node: crate::data::handle::NodeId,
    resolver: &mut impl ComparatorPolicyResolver,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<(), SignalError> {
    let prepared = PreparedEvaluation::reverted_clean_by_condition();
    crate::logic::evaluation::apply_prepared_evaluation_after_dependencies_with_policy(
        graph,
        node,
        prepared,
        resolver,
        None,
        0,
        None,
        false,
        &mut EvaluationWork::Conditional(work),
    )?;
    Ok(())
}

fn apply_deferred(
    graph: &mut SignalGraph,
    node: crate::data::handle::NodeId,
    resolver: &mut impl ComparatorPolicyResolver,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<(), SignalError> {
    let prepared = PreparedEvaluation::deferred_by_condition();
    crate::logic::evaluation::apply_prepared_evaluation_after_dependencies_with_policy(
        graph,
        node,
        prepared,
        resolver,
        None,
        0,
        None,
        false,
        &mut EvaluationWork::Conditional(work),
    )?;
    Ok(())
}
