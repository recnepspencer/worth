use std::sync::{Arc, OnceLock};

use worth_query_host::facade::admission::resource_admission::{
    WorthQueryExecutionResourceSupport, WorthQueryFixedExecutionCapacity,
};
use worth_query_host::facade::convergence_epoch::{
    WorthQueryCandidateSemanticFamilies, WorthQueryConvergenceAssessment,
    WorthQueryConvergenceComparison, WorthQueryConvergenceDomainFailure,
    WorthQueryConvergenceDomainProvider, WorthQueryConvergenceProgress,
    WorthQueryConvergenceProviderFamilies, WorthQueryConvergenceRepeatedState,
    WorthQueryIterationSemanticFamilies,
};
use worth_query_host::facade::declaration::domain_computation::{
    WorthQueryCancellationSafePointFamily, WorthQueryExecutionMode,
};
use worth_query_host::facade::domain;
use worth_query_host::facade::installed::provider_session as provider;

pub(super) fn exercise_combined_builder() {
    let _ = worth_query::facade::runtime::WorthQueryRuntimeBuilder::new(
        worth_query::facade::consumer_kit::in_memory_test_product_world_resources(),
    )
    .convergent_invariant_graph_participation_provider(ConsumerGraph, ConsumerCombinedProvider);
    let _ = worth_query::facade::runtime::WorthQueryRuntimeBuilder::new(
        worth_query::facade::consumer_kit::in_memory_test_product_world_resources(),
    )
    .atomic_convergent_invariant_graph_participation_provider(
        ConsumerGraph,
        ConsumerCombinedProvider,
        ConsumerCommit,
    );
}

struct ConsumerGraph;
struct ConsumerCommit;
struct ConsumerCombinedProvider;
struct ConsumerExecution;

impl provider::WorthQueryGraphProviderExecution for ConsumerExecution {
    fn advance(
        &mut self,
        _step: &mut provider::WorthQueryGraphProviderStep,
    ) -> Result<
        provider::WorthQueryGraphProviderStepDisposition,
        provider::WorthQueryGraphProviderFailure,
    > {
        unreachable!("consumer capability registration performs no graph execution")
    }

    fn dispose(&mut self) -> Result<(), provider::WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl provider::WorthQueryGraphParticipationProvider<ConsumerGraph> for ConsumerCombinedProvider {
    type Execution = ConsumerExecution;

    fn execution_resource_support(&self) -> WorthQueryExecutionResourceSupport {
        resource_support()
    }

    fn begin(
        &self,
        _call: &provider::WorthQueryGraphProviderCall,
        start: &mut provider::WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        provider::WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        provider::WorthQueryGraphProviderFailure,
    > {
        start
            .admit_cooperative_execution(|| ConsumerExecution)
            .map_err(|denial| provider::WorthQueryGraphProviderFailure::new(denial.detail()))
    }
}

impl provider::WorthQueryProviderSessionLifecycle for ConsumerCombinedProvider {
    fn readmit_provider_plan(
        &self,
        _plan: &provider::WorthQueryProviderExecutionPlanView<'_>,
        admission: provider::WorthQueryProviderSessionTokenAdmission,
    ) -> Result<provider::WorthQueryProviderSessionToken, provider::WorthQueryProviderSessionFailure>
    {
        admission.admit("consumer-session")
    }

    fn prepare_provider_session(
        &self,
        _session: &provider::WorthQueryProviderSessionView<'_>,
    ) -> Result<(), provider::WorthQueryProviderSessionFailure> {
        Ok(())
    }

    fn prepare_staged_session(
        &self,
        _session: &provider::WorthQueryProviderSessionView<'_>,
    ) -> Result<(), provider::WorthQueryProviderSessionFailure> {
        Ok(())
    }

    fn commit_prepared_session(
        &self,
        _session: &provider::WorthQueryProviderSessionView<'_>,
    ) -> Result<
        provider::WorthQueryProviderTerminalDescription,
        provider::WorthQueryProviderSessionCommitStop,
    > {
        Ok(provider::WorthQueryProviderTerminalDescription::new("consumer-commit").unwrap())
    }

    fn abort_provider_session(
        &self,
        _session: &provider::WorthQueryProviderSessionView<'_>,
    ) -> Result<
        provider::WorthQueryProviderTerminalDescription,
        provider::WorthQueryProviderSessionFailure,
    > {
        Ok(provider::WorthQueryProviderTerminalDescription::new("consumer-abort").unwrap())
    }
}

impl provider::WorthQueryDecisionFactProvider for ConsumerCombinedProvider {
    fn observe_decision_fact(
        &self,
        _session: provider::WorthQueryProviderSessionView<'_>,
        _request: provider::WorthQueryDecisionFactRequestView<'_>,
        admission: provider::WorthQueryDecisionFactAdmission,
    ) -> Result<provider::WorthQueryDecisionFactEvidence, provider::WorthQueryDecisionReadSetFailure>
    {
        admission.observe("consumer-v1")
    }

    fn compare_decision_fact(
        &self,
        _session: provider::WorthQueryProviderSessionView<'_>,
        evidence: provider::WorthQueryDecisionFactEvidenceView<'_>,
        admission: provider::WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<
        provider::WorthQueryDecisionFactComparisonEvidence,
        provider::WorthQueryDecisionReadSetFailure,
    > {
        admission.observe_current_version(evidence.physical_version_evidence())
    }
}

impl provider::WorthQueryProvisionalGraphProvider for ConsumerCombinedProvider {
    fn stage_provisional_overlay(
        &self,
        _session: provider::WorthQueryProviderSessionView<'_>,
        program: provider::WorthQueryProvisionalEffectProgramView<'_>,
        admission: provider::WorthQueryProvisionalOverlayAdmission,
    ) -> Result<
        provider::WorthQueryProvisionalOverlayEvidence,
        provider::WorthQueryProvisionalFailure,
    > {
        let facts = program.steps().iter().map(|step| {
            let (identity, origin) = proposed_fact(step.action());
            provider::WorthQueryProposedFact::new(identity, origin, "consumer-value").unwrap()
        });
        admission.admit("consumer-overlay", facts)
    }

    fn discard_provisional_overlay(
        &self,
        _evidence: provider::WorthQueryProvisionalOverlayEvidenceView<'_>,
    ) -> Result<(), provider::WorthQueryProvisionalFailure> {
        Ok(())
    }
}

impl provider::WorthQueryInvariantExecutionProvider for ConsumerCombinedProvider {
    fn load_invariant_state(
        &self,
        _session: provider::WorthQueryProviderSessionView<'_>,
        request: provider::WorthQueryInvariantStateLoadRequestView<'_>,
        admission: provider::WorthQueryInvariantStateLoadAdmission,
    ) -> Result<
        provider::WorthQueryInvariantStateLoadEvidence,
        provider::WorthQueryInvariantExecutionFailure,
    > {
        let count = request.locators().len();
        admission.admit(
            "consumer-load",
            request.locators().to_vec(),
            provider::WorthQueryInvariantStructuralCounters::new(count, count as u64, 0),
        )
    }

    fn execute_invariant(
        &self,
        _session: provider::WorthQueryProviderSessionView<'_>,
        execution: provider::WorthQueryBoundInvariantExecutionView<'_>,
        admission: provider::WorthQueryInvariantVerdictAdmission,
    ) -> Result<
        provider::WorthQueryInvariantProviderVerdict,
        provider::WorthQueryInvariantExecutionFailure,
    > {
        let evidence = provider::WorthQueryInvariantVerdictEvidence::new(
            execution.requirement().slot(),
            "consumer-diagnostic",
            "consumer-execution",
            1,
        )?;
        match execution.requirement().enforcement() {
            domain::WorthQueryInvariantEnforcement::Blocking => admission.passed(evidence),
            domain::WorthQueryInvariantEnforcement::Advisory => admission.advisory(evidence),
        }
    }
}

impl WorthQueryConvergenceDomainProvider for ConsumerCombinedProvider {
    fn convergence_families(&self) -> &WorthQueryConvergenceProviderFamilies {
        static FAMILIES: OnceLock<WorthQueryConvergenceProviderFamilies> = OnceLock::new();
        FAMILIES.get_or_init(|| {
            WorthQueryConvergenceProviderFamilies::new(
                WorthQueryCandidateSemanticFamilies::new(
                    "universe",
                    "termination",
                    "feasibility",
                    "comparison",
                    "incumbent",
                )
                .unwrap(),
                WorthQueryIterationSemanticFamilies::new(
                    "progress",
                    "comparator",
                    "repeated-state",
                )
                .unwrap(),
            )
        })
    }

    fn compare(
        &self,
        _assessment: &WorthQueryConvergenceAssessment<'_>,
    ) -> Result<WorthQueryConvergenceComparison, WorthQueryConvergenceDomainFailure> {
        Err(unused())
    }

    fn measure_progress(
        &self,
        _assessment: &WorthQueryConvergenceAssessment<'_>,
        _comparison: &WorthQueryConvergenceComparison,
    ) -> Result<WorthQueryConvergenceProgress, WorthQueryConvergenceDomainFailure> {
        Err(unused())
    }

    fn detect_repeated_state(
        &self,
        _assessment: &WorthQueryConvergenceAssessment<'_>,
        _comparison: &WorthQueryConvergenceComparison,
        _progress: WorthQueryConvergenceProgress,
    ) -> Result<WorthQueryConvergenceRepeatedState, WorthQueryConvergenceDomainFailure> {
        Err(unused())
    }
}

fn proposed_fact(
    action: &provider::WorthQueryProvisionalEffectAction,
) -> (&str, provider::WorthQueryProposedFactOrigin) {
    match action {
        provider::WorthQueryProvisionalEffectAction::Create { symbolic_identity } => (
            symbolic_identity,
            provider::WorthQueryProposedFactOrigin::StagedCreation,
        ),
        provider::WorthQueryProvisionalEffectAction::Replace { target_identity } => (
            target_identity,
            provider::WorthQueryProposedFactOrigin::StagedReplacement,
        ),
        provider::WorthQueryProvisionalEffectAction::Retire { target_identity } => (
            target_identity,
            provider::WorthQueryProposedFactOrigin::StagedRetirement,
        ),
        provider::WorthQueryProvisionalEffectAction::DeriveView { view_identity } => (
            view_identity,
            provider::WorthQueryProposedFactOrigin::DerivedProvisionalView,
        ),
    }
}

fn resource_support() -> WorthQueryExecutionResourceSupport {
    let envelope = domain::WorthQueryExecutionResourceEnvelope::bounded(
        8,
        8,
        WorthQueryExecutionMode::Synchronous,
        WorthQueryCancellationSafePointFamily::new("consumer-step").unwrap(),
    );
    WorthQueryExecutionResourceSupport::new(
        domain::WorthQueryExecutionProviderFamily::new("consumer-provider").unwrap(),
        domain::WorthQueryExecutionAccessProductFamily::new("consumer-access").unwrap(),
        domain::WorthQueryExecutionAllocatorFamily::new("consumer-arena").unwrap(),
        envelope,
        Arc::new(WorthQueryFixedExecutionCapacity::mint("consumer-provider", 1).unwrap()),
    )
}

fn unused() -> WorthQueryConvergenceDomainFailure {
    WorthQueryConvergenceDomainFailure::new("consumer registration does not execute convergence")
}
