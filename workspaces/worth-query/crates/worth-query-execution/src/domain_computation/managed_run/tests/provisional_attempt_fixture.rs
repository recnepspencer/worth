use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use super::decision_read_set_fixture::managed_provisional_graph_run_with_provider;
use super::*;
use crate::domain_computation::{
    WorthQueryDecisionFactAdmission, WorthQueryDecisionFactComparisonAdmission,
    WorthQueryDecisionFactComparisonEvidence, WorthQueryDecisionFactEvidence,
    WorthQueryDecisionFactEvidenceView, WorthQueryDecisionFactLocator,
    WorthQueryDecisionFactProvider, WorthQueryDecisionFactRequest,
    WorthQueryDecisionFactRequestView, WorthQueryDecisionReadSetFailure,
    WorthQueryDecisionReadSetFreshnessOutcome, WorthQueryFreshDecisionReadSet,
    WorthQueryInvariantVerdictAdmission, WorthQueryProposedFact, WorthQueryProposedFactOrigin,
    WorthQueryProviderExecutionPlanView, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionLifecycle, WorthQueryProviderSessionToken,
    WorthQueryProviderSessionTokenAdmission, WorthQueryProviderSessionView,
    WorthQueryProvisionalEffectAction, WorthQueryProvisionalEffectProgramView,
    WorthQueryProvisionalEffectStep, WorthQueryProvisionalFailure,
    WorthQueryProvisionalGraphProvider, WorthQueryProvisionalOverlayAdmission,
    WorthQueryProvisionalOverlayEvidence, WorthQueryProvisionalOverlayEvidenceView,
};
use worth_query_installation::facade::{WorthQueryDecisionFactFamily, WorthQueryDecisionFactKind};

#[derive(Default)]
pub(super) struct ProvisionalProviderState {
    pub(super) authoritative: BTreeMap<String, String>,
    pub(super) overlays: BTreeMap<String, BTreeMap<String, String>>,
    pub(super) session_overlays: BTreeMap<String, String>,
    pub(super) invariant_loaded_overlays: Vec<String>,
    pub(super) invariant_load_worlds: BTreeMap<String, (bool, bool)>,
    pub(super) stage_calls: usize,
    pub(super) discard_calls: usize,
    pub(super) discarded_session_tokens: Vec<(String, u64)>,
    pub(super) abort_calls: usize,
    pub(super) invariant_load_calls: usize,
    pub(super) invariant_execution_calls: usize,
    pub(super) discard_panics_remaining: usize,
    pub(super) invariant_outcome: InvariantFixtureOutcome,
    pub(super) provisional_stage_outcome: ProvisionalStageFixtureOutcome,
    pub(super) retained_invariant_admission: Option<WorthQueryInvariantVerdictAdmission>,
    pub(super) decision_version: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum InvariantFixtureOutcome {
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
pub(super) enum ProvisionalStageFixtureOutcome {
    #[default]
    Honest,
    OmitStagedFact,
    AddUndeclaredStagedFact,
    Panic,
}

pub(super) struct ProvisionalProvider {
    pub(super) state: Arc<Mutex<ProvisionalProviderState>>,
}

pub(super) struct UnusedProvisionalExecution;

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
        crate::domain_computation::provider_session::execution_resource_support("provisional", 8)
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
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
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
            crate::domain_computation::WorthQueryProviderTerminalDescription::new(
                "provisional commit completed",
            )
            .expect("fixture description is valid"),
        )
    }

    fn abort_provider_session(
        &self,
        _session: &WorthQueryProviderSessionView<'_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        WorthQueryProviderSessionFailure,
    > {
        self.state.lock().unwrap().abort_calls += 1;
        Ok(
            crate::domain_computation::WorthQueryProviderTerminalDescription::new(
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

pub(super) fn provisional_run(
    state: Arc<Mutex<ProvisionalProviderState>>,
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
    )
}

pub(super) fn staged_with_fresh_read_set<'run>(
    running: &'run mut WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
) -> (
    crate::domain_computation::WorthQuerySessionBoundReadsAndEffects<'run>,
    WorthQueryFreshDecisionReadSet,
) {
    let staged = running
        .admit_provider_execution_plan(graph)
        .unwrap()
        .readmit()
        .unwrap()
        .prepare()
        .unwrap()
        .bind_reads_and_effects();
    let fresh = {
        let reads = staged.read_authority();
        let receipt = reads
            .capture_decision_read_set(
                [
                    "base",
                    "old",
                    "source-1",
                    "search-1",
                    "candidate-a",
                    "transform-1",
                    "policy-1",
                    "correspondence-1",
                    "identity-map-1",
                ]
                .into_iter()
                .map(|identity| {
                    WorthQueryDecisionFactRequest::new(
                        "planning-basis",
                        WorthQueryDecisionFactLocator::observed(identity).unwrap(),
                    )
                    .unwrap()
                }),
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                None,
            )
            .unwrap();
        match reads.compare_decision_read_set(receipt).unwrap() {
            WorthQueryDecisionReadSetFreshnessOutcome::Fresh(fresh) => fresh,
            WorthQueryDecisionReadSetFreshnessOutcome::Stale(_) => {
                panic!("fixture read set must remain fresh")
            }
        }
    };
    (staged, fresh)
}

pub(super) fn effect_step(
    action: WorthQueryProvisionalEffectAction,
) -> WorthQueryProvisionalEffectStep {
    WorthQueryProvisionalEffectStep::new("mutation", action).unwrap()
}

pub(super) fn cleanup(running: WorthQueryRunningDirectRun) {
    running
        .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
        .cleanup()
        .expect("provisional fixture cleanup should complete");
}

fn proposed_change(
    action: &WorthQueryProvisionalEffectAction,
) -> (String, WorthQueryProposedFactOrigin, &'static str) {
    match action {
        WorthQueryProvisionalEffectAction::Create { symbolic_identity } => (
            symbolic_identity.to_string(),
            WorthQueryProposedFactOrigin::StagedCreation,
            "created",
        ),
        WorthQueryProvisionalEffectAction::Replace { target_identity } => (
            target_identity.to_string(),
            WorthQueryProposedFactOrigin::StagedReplacement,
            "replaced",
        ),
        WorthQueryProvisionalEffectAction::Retire { target_identity } => (
            target_identity.to_string(),
            WorthQueryProposedFactOrigin::StagedRetirement,
            "retired",
        ),
        WorthQueryProvisionalEffectAction::DeriveView { view_identity } => (
            view_identity.to_string(),
            WorthQueryProposedFactOrigin::DerivedProvisionalView,
            "derived",
        ),
    }
}
