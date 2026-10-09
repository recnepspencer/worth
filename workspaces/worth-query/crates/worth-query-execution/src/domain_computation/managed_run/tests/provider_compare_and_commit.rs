use std::sync::{Arc, Mutex};

use worth_query_installation::facade::{
    WorthQueryInstalledInvariantExecutionRequirement, WorthQueryInvariantEnforcement,
};

use super::invariant_execution::{locator, requirement};
use super::invariant_execution_fixture::invariant_run;
use super::provisional_attempt_fixture::{
    cleanup, effect_step, staged_with_fresh_read_set, ProvisionalProviderState,
};
use crate::domain_computation::{
    WorthQueryInvariantReceipt, WorthQueryProviderCommitAdmissionDenial,
    WorthQueryProviderCompareAndCommitOutcome, WorthQueryProvisionalEffectAction,
};

#[test]
fn exact_invariant_progression_is_consumed_by_provider_commit() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let state = provider_state();
        let requirements = blocking_requirements();
        let (mut running, graph) = invariant_run(Arc::clone(&state), requirements);
        let inspection = proposed_inspection(execution, &mut running, &graph);
        let receipt = execute_installed_invariant(&inspection);
        let progression = inspection
            .admit_invariant_progression([receipt])
            .expect("exact installed invariant must admit progression");
        let candidate = match inspection.bind_invariant_progression(progression) {
            Ok(candidate) => candidate,
            Err(_) => panic!("progression must bind to its exact proposed state"),
        };

        let outcome = candidate.compare_and_commit();
        let WorthQueryProviderCompareAndCommitOutcome::Committed(committed) = outcome else {
            panic!("fresh invariant-approved state must commit")
        };
        assert_eq!(
            committed.provider_description().as_str(),
            "provisional commit completed"
        );
        assert_eq!(
            state.lock().unwrap().authoritative.get("base").unwrap(),
            "replaced"
        );
        cleanup(running);
    });
}

#[test]
fn identical_provider_text_cannot_substitute_for_terminal_owner_binding() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let first = committed_provider_session(execution);
        let second = committed_provider_session(execution);

        assert_eq!(first.provider_description(), second.provider_description());
        assert!(
            !first
                .terminal_binding()
                .same_session(second.terminal_binding()),
            "equal provider-authored descriptions must not identify a terminal owner"
        );
    });
}

#[test]
fn relevant_drift_stales_before_provider_commit() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let state = provider_state();
        let (mut running, graph) = invariant_run(Arc::clone(&state), blocking_requirements());
        let inspection = proposed_inspection(execution, &mut running, &graph);
        let receipt = execute_installed_invariant(&inspection);
        let progression = inspection.admit_invariant_progression([receipt]).unwrap();
        let candidate = match inspection.bind_invariant_progression(progression) {
            Ok(candidate) => candidate,
            Err(_) => panic!("exact progression must bind"),
        };
        state.lock().unwrap().decision_version = Some("base-v2".to_owned());

        let outcome = candidate.compare_and_commit();
        let WorthQueryProviderCompareAndCommitOutcome::Stale(stale) = outcome else {
            panic!("changed decision evidence must stale")
        };
        assert_eq!(stale.stale_fact_count(), 9);
        assert_eq!(
            state.lock().unwrap().authoritative.get("base").unwrap(),
            "base-value"
        );
        cleanup(running);
    });
}

#[test]
fn equivalent_proposal_from_another_session_rejects_foreign_progression() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let first_state = provider_state();
        let (mut first_run, first_graph) =
            invariant_run(Arc::clone(&first_state), blocking_requirements());
        let first = proposed_inspection(execution, &mut first_run, &first_graph);
        let receipt = execute_installed_invariant(&first);
        let progression = first.admit_invariant_progression([receipt]).unwrap();
        first.discard();
        cleanup(first_run);

        let second_state = provider_state();
        let (mut second_run, second_graph) =
            invariant_run(Arc::clone(&second_state), blocking_requirements());
        let second = proposed_inspection(execution, &mut second_run, &second_graph);
        let (denial, second) = match second.bind_invariant_progression(progression) {
            Ok(_) => panic!("equivalent state from another session needs its own progression"),
            Err(denial) => denial,
        };
        assert_eq!(
            denial,
            WorthQueryProviderCommitAdmissionDenial::ForeignInvariantProgression
        );
        second.discard();
        cleanup(second_run);
    });
}

fn proposed_inspection<'run>(
    execution: &'run crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    running: &'run mut crate::domain_computation::WorthQueryRunningDirectRun,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
) -> crate::domain_computation::WorthQueryProposedStateInspection<'run> {
    let (staged, fresh) = staged_with_fresh_read_set(execution, running, graph);
    let program = staged
        .effect_authority()
        .lower_provisional_program(
            &fresh,
            [effect_step(WorthQueryProvisionalEffectAction::Replace {
                target_identity: "base".into(),
            })],
        )
        .unwrap();
    staged
        .begin_provisional_attempt(fresh, program)
        .unwrap()
        .materialize_proposed_state()
        .inspect()
}

fn execute_installed_invariant(
    inspection: &crate::domain_computation::WorthQueryProposedStateInspection<'_>,
) -> WorthQueryInvariantReceipt {
    inspection
        .select_installed_invariant("closed-loop")
        .unwrap()
        .admit_state_load_plan([locator("base")])
        .unwrap()
        .execute()
        .unwrap()
}

fn blocking_requirements() -> Vec<WorthQueryInstalledInvariantExecutionRequirement> {
    vec![requirement(
        "closed-loop",
        WorthQueryInvariantEnforcement::Blocking,
        4,
    )]
}

fn provider_state() -> Arc<Mutex<ProvisionalProviderState>> {
    super::invariant_execution::state()
}

fn committed_provider_session(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
) -> crate::domain_computation::WorthQueryCommittedProviderSession {
    let state = provider_state();
    let (mut running, graph) = invariant_run(Arc::clone(&state), blocking_requirements());
    let inspection = proposed_inspection(execution, &mut running, &graph);
    let receipt = execute_installed_invariant(&inspection);
    let progression = inspection.admit_invariant_progression([receipt]).unwrap();
    let candidate = inspection
        .bind_invariant_progression(progression)
        .unwrap_or_else(|_| panic!("exact progression must bind"));
    let WorthQueryProviderCompareAndCommitOutcome::Committed(committed) =
        candidate.compare_and_commit()
    else {
        panic!("fresh invariant-approved state must commit")
    };
    cleanup(running);
    committed
}
