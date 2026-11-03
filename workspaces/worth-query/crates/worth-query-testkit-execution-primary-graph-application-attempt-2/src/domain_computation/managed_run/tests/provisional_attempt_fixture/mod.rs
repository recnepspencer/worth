use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::proposed_change::proposed_change;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::domain_computation::managed_run::tests::decision_read_set_fixture::managed_provisional_graph_run_with_provider;
use crate::domain_computation::managed_run::tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactComparisonAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactComparisonEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactEvidenceView;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactLocator;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactProvider;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactRequest;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionFactRequestView;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetFailure;
use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetFreshnessOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryFreshDecisionReadSet;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantVerdictAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryProposedFact;
use worth_query_execution::facade::domain_computation::WorthQueryProposedFactOrigin;
use worth_query_execution::facade::domain_computation::WorthQueryProviderExecutionPlanView;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionLifecycle;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionToken;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionTokenAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionView;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalEffectAction;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalEffectProgramView;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalEffectStep;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalFailure;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalGraphProvider;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalOverlayAdmission;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalOverlayEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryProvisionalOverlayEvidenceView;
use worth_query_installation::facade::WorthQueryDecisionFactFamily;
use worth_query_installation::facade::WorthQueryDecisionFactKind;

#[derive(Default)]
pub struct ProvisionalProviderState {
    pub authoritative: BTreeMap<String, String>,
    pub overlays: BTreeMap<String, BTreeMap<String, String>>,
    pub session_overlays: BTreeMap<String, String>,
    pub invariant_loaded_overlays: Vec<String>,
    pub invariant_load_worlds: BTreeMap<String, (bool, bool)>,
    pub stage_calls: usize,
    pub discard_calls: usize,
    pub discarded_session_tokens: Vec<(String, u64)>,
    pub abort_calls: usize,
    pub invariant_load_calls: usize,
    pub invariant_execution_calls: usize,
    pub discard_panics_remaining: usize,
    pub invariant_outcome: InvariantFixtureOutcome,
    pub provisional_stage_outcome: ProvisionalStageFixtureOutcome,
    pub retained_invariant_admission: Option<WorthQueryInvariantVerdictAdmission>,
    pub decision_version: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InvariantFixtureOutcome {
    #[default]
    Auto,
    EmptyLoad,
    OmitLoad,
    OverreadLoad,
    Indeterminate,
    Exhausted,
    RetainThenSubstitute,
    PanicDuringLoad,
    PanicDuringExecution,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProvisionalStageFixtureOutcome {
    #[default]
    Honest,
    OmitStagedFact,
    AddUndeclaredStagedFact,
    Panic,
}

pub struct ProvisionalProvider {
    pub state: Arc<Mutex<ProvisionalProviderState>>,
}

pub struct UnusedProvisionalExecution;

impl WorthQueryGraphProviderExecution for UnusedProvisionalExecution {
    fn advance(
        &mut self,
        _step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        unreachable!("provisional tests use the sealed provider session")
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for ProvisionalProvider {
    type Execution = UnusedProvisionalExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::tests::execution_resource_support(
            "provisional",
            8,
        )
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        unreachable!("provisional tests use the sealed provider session")
    }
}

impl WorthQueryProviderSessionLifecycle for ProvisionalProvider {
    fn readmit_provider_plan(
        &self,
        _plan: &WorthQueryProviderExecutionPlanView<'_>,
        admission: WorthQueryProviderSessionTokenAdmission,
    ) -> Result<WorthQueryProviderSessionToken, WorthQueryProviderSessionFailure> {
        admission.admit("provisional-session")
    }

    fn prepare_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        Ok(())
    }

    fn prepare_staged_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<(), WorthQueryProviderSessionFailure> {
        Ok(())
    }

    fn commit_prepared_session(
        &self,
        session: &WorthQueryProviderSessionView<'_>,
        _allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription,
        worth_query_execution::facade::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        let mut state = self.state.lock().unwrap();
        let overlay_identity = state
            .session_overlays
            .remove(session.identity())
            .expect("commit requires the exact session-owned overlay");
        state.authoritative = state
            .overlays
            .remove(&overlay_identity)
            .expect("prepared overlay must remain available through commit");
        Ok(
            worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription::new(
                "provisional commit completed",
            )
            .expect("fixture description is valid"),
        )
    }

    fn abort_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        self.state.lock().unwrap().abort_calls += 1;
        Ok(
            worth_query_execution::facade::domain_computation::WorthQueryProviderTerminalDescription::new(
                "provisional abort",
            )
            .expect("fixture description is valid"),
        )
    }
}

impl WorthQueryDecisionFactProvider for ProvisionalProvider {
    fn observe_decision_fact(
        &self,
        _session: WorthQueryProviderSessionView<'_>,
        _request: WorthQueryDecisionFactRequestView<'_>,
        admission: WorthQueryDecisionFactAdmission,
    ) -> Result<WorthQueryDecisionFactEvidence, WorthQueryDecisionReadSetFailure> {
        let state = self.state.lock().unwrap();
        admission.observe(state.decision_version.as_deref().unwrap_or("base-v1"))
    }

    fn compare_decision_fact(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _session: WorthQueryProviderSessionView<'_>,
        _evidence: WorthQueryDecisionFactEvidenceView<'_>,
        admission: WorthQueryDecisionFactComparisonAdmission,
    ) -> Result<WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionReadSetFailure> {
        let state = self.state.lock().unwrap();
        admission.observe_current_version(state.decision_version.as_deref().unwrap_or("base-v1"))
    }
}

impl WorthQueryProvisionalGraphProvider for ProvisionalProvider {
    fn stage_provisional_overlay(
        &self,
        session: WorthQueryProviderSessionView<'_>,
        program: WorthQueryProvisionalEffectProgramView<'_>,
        admission: WorthQueryProvisionalOverlayAdmission,
    ) -> Result<WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalFailure> {
        if self.state.lock().unwrap().provisional_stage_outcome
            == ProvisionalStageFixtureOutcome::Panic
        {
            panic!("fixture provisional staging panic");
        }
        let mut state = self.state.lock().unwrap();
        state.stage_calls += 1;
        let physical = format!("overlay-{}-{}", program.generation(), state.stage_calls);
        let mut facts = state
            .authoritative
            .iter()
            .map(|(identity, value)| {
                (
                    identity.clone(),
                    WorthQueryProposedFact::new(
                        identity.clone(),
                        WorthQueryProposedFactOrigin::AuthoritativeBase,
                        value.clone(),
                    )
                    .unwrap(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        for step in program.steps() {
            let (identity, origin, value) = proposed_change(step.action());
            facts.insert(
                identity.clone(),
                WorthQueryProposedFact::new(identity, origin, value).unwrap(),
            );
        }
        match state.provisional_stage_outcome {
            ProvisionalStageFixtureOutcome::Honest => {}
            ProvisionalStageFixtureOutcome::OmitStagedFact => {
                if let Some(step) = program.steps().first() {
                    facts.remove(proposed_change(step.action()).0.as_str());
                }
            }
            ProvisionalStageFixtureOutcome::AddUndeclaredStagedFact => {
                facts.insert(
                    "invented".to_owned(),
                    WorthQueryProposedFact::new(
                        "invented",
                        WorthQueryProposedFactOrigin::StagedCreation,
                        "invented",
                    )
                    .unwrap(),
                );
            }
            ProvisionalStageFixtureOutcome::Panic => {
                unreachable!("panic outcome exits before provider staging")
            }
        }
        state.overlays.insert(
            physical.clone(),
            facts
                .iter()
                .map(|(identity, fact)| (identity.clone(), fact.semantic_value().to_owned()))
                .collect(),
        );
        state
            .session_overlays
            .insert(session.identity().to_owned(), physical.clone());
        admission.admit(physical, facts.into_values())
    }

    fn discard_provisional_overlay(
        &self,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
        evidence: WorthQueryProvisionalOverlayEvidenceView<'_>,
    ) -> Result<(), WorthQueryProvisionalFailure> {
        let mut state = self.state.lock().unwrap();
        state.discard_calls += 1;
        state.discarded_session_tokens.push((
            evidence.token_identity().to_owned(),
            evidence.token_generation(),
        ));
        if state.discard_panics_remaining > 0 {
            state.discard_panics_remaining -= 1;
            drop(state);
            panic!("fixture provisional discard panic");
        }
        state.overlays.remove(evidence.physical_overlay_identity());
        state
            .session_overlays
            .retain(|_, overlay| overlay != evidence.physical_overlay_identity());
        Ok(())
    }
}

pub fn provisional_run(
    state: Arc<Mutex<ProvisionalProviderState>>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
) {
    managed_provisional_graph_run_with_provider(
        ProvisionalProvider { state },
        vec![WorthQueryDecisionFactFamily::new(
            "planning-basis",
            WorthQueryDecisionFactKind::ObservedValue,
        )
        .unwrap()
        .with_exact_fact_count(9)
        .unwrap()],
        resource_request,
    )
}

pub use crate::domain_computation::managed_run::tests::provisional_attempt_fixture::staged_with_fresh_read_set::staged_with_fresh_read_set;

pub fn effect_step(action: WorthQueryProvisionalEffectAction) -> WorthQueryProvisionalEffectStep {
    WorthQueryProvisionalEffectStep::new("mutation", action).unwrap()
}

pub fn cleanup(running: WorthQueryRunningDirectRun) {
    running
        .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
        .cleanup()
        .expect("provisional fixture cleanup should complete");
}

#[path = "proposed_change/mod.rs"]
pub mod proposed_change;
#[path = "staged_with_fresh_read_set/mod.rs"]
pub mod staged_with_fresh_read_set;
