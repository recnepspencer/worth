use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::output::NodeEvaluationResult;
use worth_proof::{ExecuteReadyRecipeTransition, Transition};

mod application;
mod preparation;
mod unwind;
pub(crate) mod work;
pub(crate) use unwind::SignalConditionalAttemptOutcome;

use super::artifact_reuse::{resolve_artifact_reuse, SignalConditionalArtifactReuseObservation};
use super::dependency_versions::{record_dependency_versions, SignalConditionalDependencyVersions};
use super::execution_proof::SignalConditionalExecutedRecipe;
use super::{
    InstalledSignalConditionResolver, InstalledSignalConditionalContract,
    SignalConditionalDecisionClass, SignalConditionalDecisionCounters,
    SignalConditionalDecisionEvidence,
};
use application::{apply_passive, ConditionalResolutionAttempt};
use preparation::{prepare_conditional_attempt, PreparedConditionalAttempt};

pub struct SignalConditionalExecutionRequest<'a> {
    pub(super) contract: &'a InstalledSignalConditionalContract,
    pub(super) snapshot_identity: &'a str,
    pub(super) execution_identity: &'a str,
    pub(super) attempt: u64,
    pub(super) force_on_demand: bool,
}

#[derive(Debug)]
pub struct SignalConditionalExecutionFailure {
    error: SignalError,
    counters: SignalConditionalDecisionCounters,
}

impl SignalConditionalExecutionFailure {
    pub const fn counters(&self) -> SignalConditionalDecisionCounters {
        self.counters
    }

    pub fn into_error(self) -> SignalError {
        self.error
    }
}

impl<'a> SignalConditionalExecutionRequest<'a> {
    pub fn new(
        contract: &'a InstalledSignalConditionalContract,
        snapshot_identity: &'a str,
        execution_identity: &'a str,
        attempt: u64,
    ) -> Self {
        Self {
            contract,
            snapshot_identity,
            execution_identity,
            attempt,
            force_on_demand: false,
        }
    }

    pub fn force_on_demand(mut self) -> Self {
        self.force_on_demand = true;
        self
    }

    pub(crate) fn contract(&self) -> &InstalledSignalConditionalContract {
        self.contract
    }
}

impl SignalGraph {
    pub fn execute_installed_conditional(
        &mut self,
        request: SignalConditionalExecutionRequest<'_>,
        condition_resolver: &mut impl InstalledSignalConditionResolver,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        compute: impl FnOnce() -> Result<NodeEvaluationResult, SignalError>,
    ) -> Result<SignalConditionalDecisionEvidence, SignalConditionalExecutionFailure> {
        let mut work = crate::data::retained_storage::RetainedStoragePreparation::new(
            self.installed_runtime_policy()
                .conditional_evaluation_budget()
                .maximum_attempt_visits,
        );
        self.execute_installed_conditional_attempt(
            request,
            condition_resolver,
            comparator_resolver,
            compute,
            &mut work,
        )
        .resume()
    }

    pub(crate) fn execute_installed_conditional_attempt(
        &mut self,
        request: SignalConditionalExecutionRequest<'_>,
        condition_resolver: &mut impl InstalledSignalConditionResolver,
        comparator_resolver: &mut impl ComparatorPolicyResolver,
        compute: impl FnOnce() -> Result<NodeEvaluationResult, SignalError>,
        work: &mut crate::data::retained_storage::RetainedStoragePreparation,
    ) -> SignalConditionalAttemptOutcome {
        let mut counters = SignalConditionalDecisionCounters::default();
        let providers = ConditionalExecutionProviders {
            condition: condition_resolver,
            comparator: comparator_resolver,
            compute: Some(compute),
        };
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            execute_conditional_attempt(self, request, providers, &mut counters, work)
        })) {
            Ok(result) => SignalConditionalAttemptOutcome::Completed(
                result.map_err(|error| SignalConditionalExecutionFailure { error, counters }),
            ),
            Err(payload) => SignalConditionalAttemptOutcome::Unwound { payload, counters },
        }
    }
}

struct ConditionalExecutionProviders<'a, Condition, Comparator, Compute> {
    condition: &'a mut Condition,
    comparator: &'a mut Comparator,
    compute: Option<Compute>,
}

struct ConditionalExecutionCompletion {
    dependency_versions: SignalConditionalDependencyVersions,
    ready: super::execution_proof::SignalConditionalReadyRecipe,
    node: crate::data::handle::NodeId,
    output_aspect: crate::data::aspect::Aspect,
    output_version_before: u64,
    dependency_changed: bool,
}

struct ConditionalFinalization<'a> {
    request: SignalConditionalExecutionRequest<'a>,
    completion: ConditionalExecutionCompletion,
    class: SignalConditionalDecisionClass,
}

fn execute_conditional_attempt<Condition, Comparator, Compute>(
    graph: &mut SignalGraph,
    request: SignalConditionalExecutionRequest<'_>,
    mut providers: ConditionalExecutionProviders<'_, Condition, Comparator, Compute>,
    counters: &mut SignalConditionalDecisionCounters,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<SignalConditionalDecisionEvidence, SignalError>
where
    Condition: InstalledSignalConditionResolver,
    Comparator: ComparatorPolicyResolver,
    Compute: FnOnce() -> Result<NodeEvaluationResult, SignalError>,
{
    let prepared =
        prepare_conditional_attempt(graph, &request, providers.comparator, counters, work)?;
    let PreparedConditionalAttempt {
        dependency_versions,
        ready,
        node,
        output_aspect,
        output_version_before,
        dependency_changed,
        passive_dependency_hit,
        ready_invalidation,
    } = prepared;
    let class = if passive_dependency_hit {
        counters.application_contacts += 1;
        apply_passive(graph, node, providers.comparator, work)?;
        SignalConditionalDecisionClass::DependencyUnchanged
    } else {
        counters.condition_checks += 1;
        ConditionalResolutionAttempt {
            graph,
            request: &request,
            providers: &mut providers,
            ready_invalidation,
            counters,
            work,
        }
        .resolve()?
    };
    finalize_conditional_attempt(
        graph,
        ConditionalFinalization {
            request,
            completion: ConditionalExecutionCompletion {
                dependency_versions,
                ready,
                node,
                output_aspect,
                output_version_before,
                dependency_changed,
            },
            class,
        },
        providers.comparator,
        counters,
        work,
    )
}

fn finalize_conditional_attempt(
    graph: &mut SignalGraph,
    finalization: ConditionalFinalization<'_>,
    comparator: &mut impl ComparatorPolicyResolver,
    counters: &mut SignalConditionalDecisionCounters,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<SignalConditionalDecisionEvidence, SignalError> {
    work::reserve(work, Some(1))?;
    let ConditionalFinalization {
        request,
        completion,
        class,
    } = finalization;
    retain_outcome_counter(class, counters);
    counters.output_version_reads += 1;
    let output_version_after = graph.conditional_node_version_for_scope(
        completion.node,
        completion.output_aspect,
        None,
        work,
    )?;
    let artifact_reuse_admitted = resolve_artifact_reuse(
        SignalConditionalArtifactReuseObservation {
            policy: request.contract.artifact_reuse(),
            class,
            dependency_changed: completion.dependency_changed,
            aspect: completion.output_aspect,
            before: completion.output_version_before,
            after: output_version_after,
        },
        comparator,
        counters,
    )?;
    if !matches!(
        class,
        SignalConditionalDecisionClass::DeferredByCondition
            | SignalConditionalDecisionClass::DeferredTemporal
            | SignalConditionalDecisionClass::DeferredOnDemand
    ) {
        record_dependency_versions(graph, request.contract, work)?;
    }
    let executed = ExecuteReadyRecipeTransition
        .transition(completion.ready)
        .into_value();
    let mut evidence = mint_evidence(
        request,
        ConditionalDecisionOutcome {
            class,
            counters: *counters,
            artifact_reuse_admitted,
            dependency_versions: completion.dependency_versions,
            executed,
            output_aspect: completion.output_aspect,
            output_version: output_version_after,
        },
        work,
    )?;
    counters.decisions_delivered += 1;
    evidence.counters = *counters;
    Ok(evidence)
}

fn retain_outcome_counter(
    class: SignalConditionalDecisionClass,
    counters: &mut SignalConditionalDecisionCounters,
) {
    match class {
        SignalConditionalDecisionClass::ComputedRevertedClean => {
            counters.reverted_clean_outcomes += 1;
        }
        SignalConditionalDecisionClass::DeferredByCondition => {
            counters.condition_deferrals += 1;
        }
        SignalConditionalDecisionClass::DeferredTemporal => {
            counters.temporal_deferrals += 1;
        }
        SignalConditionalDecisionClass::DeferredOnDemand => {
            counters.on_demand_deferrals += 1;
        }
        SignalConditionalDecisionClass::ComputedChanged
        | SignalConditionalDecisionClass::DependencyUnchanged
        | SignalConditionalDecisionClass::SuppressedBeforeCompute => {}
    }
}

struct ConditionalDecisionOutcome {
    class: SignalConditionalDecisionClass,
    counters: SignalConditionalDecisionCounters,
    artifact_reuse_admitted: bool,
    dependency_versions: SignalConditionalDependencyVersions,
    executed: SignalConditionalExecutedRecipe,
    output_aspect: crate::data::aspect::Aspect,
    output_version: u64,
}

fn mint_evidence(
    request: SignalConditionalExecutionRequest<'_>,
    outcome: ConditionalDecisionOutcome,
    work: &mut crate::data::retained_storage::RetainedStoragePreparation,
) -> Result<SignalConditionalDecisionEvidence, SignalError> {
    let projection_basis = super::identity::decision_projection_basis(
        request.contract,
        request.snapshot_identity,
        request.execution_identity,
        request.attempt,
        outcome.class,
        &outcome.dependency_versions,
        work,
    )?;
    let (authority, projection) =
        super::identity::mint_signal_conditional_decision_identity(projection_basis);
    Ok(SignalConditionalDecisionEvidence {
        _authority: authority,
        projection,
        contract_authority: std::sync::Arc::clone(&request.contract.authority),
        attempt: request.attempt,
        class: outcome.class,
        counters: outcome.counters,
        artifact_reuse_admitted: outcome.artifact_reuse_admitted,
        output_aspect: outcome.output_aspect,
        output_version: outcome.output_version,
        _dependency_versions: outcome.dependency_versions,
        _execution: outcome.executed,
    })
}
