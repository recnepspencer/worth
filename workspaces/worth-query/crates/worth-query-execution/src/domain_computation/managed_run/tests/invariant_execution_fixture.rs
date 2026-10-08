use std::sync::{Arc, Mutex};

use super::decision_read_set_fixture::managed_invariant_graph_run_with_provider;
use super::provisional_attempt_fixture::{
    cleanup, effect_step, staged_with_fresh_read_set, InvariantFixtureOutcome, ProvisionalProvider,
    ProvisionalProviderState,
};
use super::*;
use crate::domain_computation::{
    WorthQueryBoundInvariantExecutionView, WorthQueryInvariantExecutionDenialKind,
    WorthQueryInvariantExecutionFailure, WorthQueryInvariantExecutionProvider,
    WorthQueryInvariantProviderVerdict, WorthQueryInvariantReceipt,
    WorthQueryInvariantStateLoadAdmission, WorthQueryInvariantStateLoadEvidence,
    WorthQueryInvariantStateLoadRequestView, WorthQueryInvariantStateLocator,
    WorthQueryInvariantStructuralCounters, WorthQueryInvariantVerdictAdmission,
    WorthQueryInvariantVerdictEvidence, WorthQueryProvisionalEffectAction,
};
use worth_query_installation::facade::{
    WorthQueryDecisionFactFamily, WorthQueryDecisionFactKind,
    WorthQueryInstalledInvariantExecutionRequirement, WorthQueryInvariantEnforcement,
};

impl WorthQueryInvariantExecutionProvider for ProvisionalProvider {
    fn load_invariant_state(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        request: WorthQueryInvariantStateLoadRequestView<'_>,
        admission: WorthQueryInvariantStateLoadAdmission,
    ) -> Result<WorthQueryInvariantStateLoadEvidence, WorthQueryInvariantExecutionFailure> {
        if self.state.lock().unwrap().invariant_outcome == InvariantFixtureOutcome::PanicDuringLoad
        {
            panic!("fixture invariant load panic");
        }
        let mut state = self.state.lock().unwrap();
        state.invariant_load_calls += 1;
        if state.invariant_outcome == InvariantFixtureOutcome::EmptyLoad {
            return admission.admit(
                "empty-invariant-load",
                Vec::<WorthQueryInvariantStateLocator>::new(),
                WorthQueryInvariantStructuralCounters::new(0, 0, 0),
            );
        }
        let mut loaded = request.locators().to_vec();
        if state.invariant_outcome == InvariantFixtureOutcome::OmitLoad {
            loaded.pop();
        } else if state.invariant_outcome == InvariantFixtureOutcome::OverreadLoad {
            loaded.push(WorthQueryInvariantStateLocator::new("region", "outside").unwrap());
        }
        let physical_load = format!("invariant-load-{}", state.invariant_load_calls);
        let active_overlay = state
            .session_overlays
            .get(session.identity())
            .cloned()
            .expect("state load requires the exact live session-owned overlay");
        let overlay = state
            .overlays
            .get(&active_overlay)
            .expect("the live session overlay must remain available");
        let incremental_invalid = request.locators().iter().any(|locator| {
            overlay
                .get(locator.identity())
                .is_some_and(|value| value == "corrupt")
        });
        state.invariant_load_worlds.insert(
            physical_load.clone(),
            (incremental_invalid, incremental_invalid),
        );
        state.invariant_loaded_overlays.push(active_overlay);
        let loaded_count = loaded.len();
        admission.admit(
            physical_load,
            loaded,
            WorthQueryInvariantStructuralCounters::new(loaded_count, loaded_count as u64, 0),
        )
    }

    fn execute_invariant(
        &self,
        _session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        execution: WorthQueryBoundInvariantExecutionView<'_>,
        admission: WorthQueryInvariantVerdictAdmission,
    ) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
        if self.state.lock().unwrap().invariant_outcome
            == InvariantFixtureOutcome::PanicDuringExecution
        {
            panic!("fixture invariant execution panic");
        }
        let mut state = self.state.lock().unwrap();
        state.invariant_execution_calls += 1;
        let evidence = WorthQueryInvariantVerdictEvidence::new(
            execution.requirement().slot(),
            "fixture-diagnostic",
            format!("invariant-execution-{}", state.invariant_execution_calls),
            1,
        )?;
        match state.invariant_outcome {
            InvariantFixtureOutcome::Indeterminate => admission.indeterminate(evidence),
            InvariantFixtureOutcome::Exhausted => admission.exhausted(evidence),
            InvariantFixtureOutcome::RetainThenSubstitute => {
                retained_verdict(&mut state, admission, evidence)
            }
            InvariantFixtureOutcome::EmptyLoad
            | InvariantFixtureOutcome::OmitLoad
            | InvariantFixtureOutcome::OverreadLoad
            | InvariantFixtureOutcome::PanicDuringLoad
            | InvariantFixtureOutcome::PanicDuringExecution => {
                unreachable!("invalid loads are rejected before execution")
            }
            InvariantFixtureOutcome::Auto => {
                automatic_verdict(&state, execution, admission, evidence)
            }
        }
    }
}

pub(super) struct InvariantExecutionObservation {
    pub(super) result: Result<WorthQueryInvariantReceipt, WorthQueryInvariantExecutionFailure>,
    pub(super) full_invalid: bool,
}

pub(super) fn execute_invariant(
    state: Arc<Mutex<ProvisionalProviderState>>,
    requirements: Vec<WorthQueryInstalledInvariantExecutionRequirement>,
    slot: &str,
    locators: impl IntoIterator<Item = WorthQueryInvariantStateLocator>,
) -> InvariantExecutionObservation {
    execute_invariant_with_request(state, requirements, slot, locators, None)
}

pub(super) fn execute_invariant_with_request(
    state: Arc<Mutex<ProvisionalProviderState>>,
    requirements: Vec<WorthQueryInstalledInvariantExecutionRequirement>,
    slot: &str,
    locators: impl IntoIterator<Item = WorthQueryInvariantStateLocator>,
    request: Option<
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    >,
) -> InvariantExecutionObservation {
    let (mut running, graph) = invariant_run(state, requirements);
    let (staged, fresh) = staged_with_fresh_read_set(&mut running, &graph);
    let program = staged
        .effect_authority()
        .lower_provisional_program(
            &fresh,
            [effect_step(WorthQueryProvisionalEffectAction::Replace {
                target_identity: "base".into(),
            })],
        )
        .unwrap();
    let inspection = staged
        .begin_provisional_attempt(fresh, program)
        .unwrap()
        .materialize_proposed_state()
        .inspect();
    let full_invalid = inspection
        .facts()
        .iter()
        .any(|fact| fact.semantic_value() == "corrupt");
    let result = inspection
        .select_installed_invariant(slot)
        .and_then(|selected| selected.admit_state_load_plan(locators, request))
        .and_then(|bound| bound.execute());
    inspection.discard();
    cleanup(running);
    InvariantExecutionObservation {
        result,
        full_invalid,
    }
}

pub(super) fn admit_progression(
    state: Arc<Mutex<ProvisionalProviderState>>,
    requirements: Vec<WorthQueryInstalledInvariantExecutionRequirement>,
    slots: impl IntoIterator<Item = &'static str>,
) -> Result<
    crate::domain_computation::WorthQueryInvariantProgressionAuthority,
    WorthQueryInvariantExecutionFailure,
> {
    let (mut running, graph) = invariant_run(state, requirements);
    let (staged, fresh) = staged_with_fresh_read_set(&mut running, &graph);
    let program = staged
        .effect_authority()
        .lower_provisional_program(
            &fresh,
            [effect_step(WorthQueryProvisionalEffectAction::Replace {
                target_identity: "base".into(),
            })],
        )
        .unwrap();
    let inspection = staged
        .begin_provisional_attempt(fresh, program)
        .unwrap()
        .materialize_proposed_state()
        .inspect();
    let receipts = slots
        .into_iter()
        .map(|slot| {
            inspection
                .select_installed_invariant(slot)?
                .admit_state_load_plan(
                    [WorthQueryInvariantStateLocator::new("region", "base")?],
                    None,
                )?
                .execute()
        })
        .collect::<Result<Vec<_>, _>>();
    let progression =
        receipts.and_then(|receipts| inspection.admit_invariant_progression(receipts));
    inspection.discard();
    cleanup(running);
    progression
}

pub(super) fn invariant_run(
    state: Arc<Mutex<ProvisionalProviderState>>,
    invariants: Vec<WorthQueryInstalledInvariantExecutionRequirement>,
) -> (
    WorthQueryRunningDirectRun,
    WorthQueryInstalledGraphParticipationAuthority,
) {
    managed_invariant_graph_run_with_provider(
        ProvisionalProvider { state },
        vec![WorthQueryDecisionFactFamily::new(
            "planning-basis",
            WorthQueryDecisionFactKind::ObservedValue,
        )
        .unwrap()
        .with_exact_fact_count(9)
        .unwrap()],
        invariants,
    )
}

fn automatic_verdict(
    state: &ProvisionalProviderState,
    execution: WorthQueryBoundInvariantExecutionView<'_>,
    admission: WorthQueryInvariantVerdictAdmission,
    evidence: WorthQueryInvariantVerdictEvidence,
) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
    let (full_invalid, incremental_invalid) = state
        .invariant_load_worlds
        .get(execution.state_load_evidence().physical_load_evidence())
        .copied()
        .expect("execution must consume its exact admitted state load");
    if full_invalid != incremental_invalid {
        admission.indeterminate(evidence)
    } else if full_invalid {
        admission.violated(evidence)
    } else if execution.requirement().enforcement() == WorthQueryInvariantEnforcement::Advisory {
        admission.advisory(evidence)
    } else {
        admission.passed(evidence)
    }
}

fn retained_verdict(
    state: &mut ProvisionalProviderState,
    admission: WorthQueryInvariantVerdictAdmission,
    evidence: WorthQueryInvariantVerdictEvidence,
) -> Result<WorthQueryInvariantProviderVerdict, WorthQueryInvariantExecutionFailure> {
    if let Some(retained) = state.retained_invariant_admission.take() {
        retained.passed(evidence)
    } else {
        state.retained_invariant_admission = Some(admission);
        Err(WorthQueryInvariantExecutionFailure::new(
            WorthQueryInvariantExecutionDenialKind::ProviderRejected,
            "fixture retained the first verdict admission",
        ))
    }
}
