use std::sync::Arc;

use super::invariant_execution::{locator, requirement, state};
use super::invariant_execution_fixture::invariant_run;
use super::provisional_attempt_fixture::{cleanup, effect_step, staged_with_fresh_read_set};
use crate::domain_computation::{
    WorthQueryInvariantExecutionDenialKind, WorthQueryInvariantReceipt,
    WorthQueryProvisionalEffectAction,
};
use worth_query_installation::facade::WorthQueryInvariantEnforcement;

#[test]
fn interleaved_sessions_load_their_own_provisional_overlays() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let state = state();
        let requirements = vec![requirement(
            "closed-loop",
            WorthQueryInvariantEnforcement::Blocking,
            4,
        )];
        let (mut first_run, first_graph) = invariant_run(Arc::clone(&state), requirements.clone());
        let (mut second_run, second_graph) = invariant_run(Arc::clone(&state), requirements);

        let (first, first_token) = proposed_inspection(execution, &mut first_run, &first_graph);
        let (second, second_token) = proposed_inspection(execution, &mut second_run, &second_graph);

        let first_receipt = first
            .select_installed_invariant("closed-loop")
            .unwrap()
            .admit_state_load_plan([locator("base")])
            .unwrap()
            .execute()
            .unwrap();
        assert!(matches!(
            &first_receipt,
            WorthQueryInvariantReceipt::Passed(_)
        ));
        assert_eq!(
            state.lock().unwrap().invariant_loaded_overlays,
            ["overlay-1-1"]
        );
        let substitution = second
            .admit_invariant_progression([first_receipt])
            .expect_err("an equivalent proposed state from another session needs its own receipt");
        assert_eq!(
            substitution.kind(),
            WorthQueryInvariantExecutionDenialKind::VerdictPostureMismatch
        );

        let second_receipt = second
            .select_installed_invariant("closed-loop")
            .unwrap()
            .admit_state_load_plan([locator("base")])
            .unwrap()
            .execute()
            .unwrap();
        assert!(matches!(
            &second_receipt,
            WorthQueryInvariantReceipt::Passed(_)
        ));
        second
            .admit_invariant_progression([second_receipt])
            .expect("the exact session receipt should progress");
        assert_eq!(
            state.lock().unwrap().invariant_loaded_overlays,
            ["overlay-1-1", "overlay-1-2"]
        );

        first.discard();
        second.discard();
        assert_eq!(
            state.lock().unwrap().discarded_session_tokens,
            [first_token, second_token]
        );
        cleanup(first_run);
        cleanup(second_run);
    });
}

fn proposed_inspection<'run>(
    execution: &'run crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    running: &'run mut super::WorthQueryRunningDirectRun,
    graph: &worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
) -> (
    crate::domain_computation::WorthQueryProposedStateInspection<'run>,
    (String, u64),
) {
    let (staged, fresh) = staged_with_fresh_read_set(execution, running, graph);
    let token = (
        staged.token_identity().to_owned(),
        staged.token_generation(),
    );
    let program = staged
        .effect_authority()
        .lower_provisional_program(
            &fresh,
            [effect_step(WorthQueryProvisionalEffectAction::Replace {
                target_identity: "base".into(),
            })],
        )
        .unwrap();
    (
        staged
            .begin_provisional_attempt(fresh, program)
            .unwrap()
            .materialize_proposed_state()
            .inspect(),
        token,
    )
}
