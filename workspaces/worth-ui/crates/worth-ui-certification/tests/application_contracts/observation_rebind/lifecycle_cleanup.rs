use worth_ui::facade::rebind::{
    UiRebindOutcome, UiRebindReconciliation, UiRebindReconciliationRequest,
    UiRebindRecoveryDenialCause, UiRebindRecoveryOutcome,
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationMode,
    UiMountedSurfaceReconciliationBinding, UiPresentationDeadline,
};

use super::support::RebindExecutionWorld;
use crate::mounted_application_lifecycle::known_empty_surface_world::profile;
use crate::mounted_host_protocol::scripted_host::{
    presented_completion, ScriptedSurfaceCompletion,
};

pub(crate) fn prove_all_projection_safe_point_cleanup() {
    timeout_cancellation_duplicate_and_supersession_are_terminal_and_clean();
    in_flight_rebind_remains_owned_until_exact_completion();
    indeterminate_rebind_retains_recovery_authority_until_shutdown_disposal();
    indeterminate_rebind_recovers_through_current_frame_reconciliation();
    recovery_rejection_retains_the_exact_retry_progression();
    recovery_completion_and_reindeterminacy_remain_managed();
    completion_disposal_and_drop_cancel_the_inherited_attempt();
    recovery_completion_disposal_retains_the_recovery_request();
    dropped_recovery_completion_cancels_before_shutdown();
}

#[test]
fn timeout_cancellation_duplicate_and_supersession_are_terminal_and_clean() {
    super::terminal_outcomes::prove_terminal_outcome_cleanup();
}

#[test]
fn in_flight_rebind_remains_owned_until_exact_completion() {
    let mut world = RebindExecutionWorld::new("phase-312-rebind-in-flight");
    world.host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending, presented_completion()],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    let prepared = world.prepare_changed();
    let candidate = prepared.candidate_generation().clone();

    let completion = match prepared.execute(1) {
        UiRebindOutcome::InFlight(completion) => completion,
        _ => panic!("pending host work must return managed completion authority"),
    };
    let completion = match completion.complete(2) {
        UiRebindOutcome::InFlight(completion) => completion,
        _ => panic!("pending completion must remain managed"),
    };
    let receipt = match completion.complete(3) {
        UiRebindOutcome::Published(receipt) => receipt,
        _ => panic!("presented completion must publish"),
    };
    assert_eq!(receipt.active_generation(), &candidate);
    drop(receipt);
    world.close();
}

#[test]
fn indeterminate_rebind_retains_recovery_authority_until_shutdown_disposal() {
    let mut world = RebindExecutionWorld::new("phase-312-rebind-indeterminate");
    let predecessor = world.session.generation_identity().clone();
    let publication = world.session.current_mounted_publication().cloned();
    world.host.push_presentation(
        worth_ui_runtime::certification_support::ScriptedPresentationOutcome::
            PresentationIndeterminate,
    );
    let prepared = world.prepare_changed();

    let recovery = match prepared.execute(1) {
        UiRebindOutcome::Indeterminate(recovery) => recovery,
        _ => panic!("uncertain host effects must return recovery authority"),
    };
    let _ = recovery.frame().cost_report();
    let session = recovery.into_session_for_shutdown();
    assert_eq!(session.generation_identity(), &predecessor);
    assert_eq!(session.current_mounted_publication(), publication.as_ref());
    let _ = session;
    world.close();
}

#[test]
fn indeterminate_rebind_recovers_through_current_frame_reconciliation() {
    let mut world = RebindExecutionWorld::new("phase-312-rebind-recovered");
    let predecessor = world
        .session
        .current_mounted_publication()
        .expect("fixture publishes a predecessor")
        .clone();
    let host = world.host.clone();
    let mut reconciliation = indeterminate_reconciliation(&mut world);
    let request = rebound_request(&mut reconciliation);
    host.push_presented();

    let receipt = match reconciliation.present_current(request, 2) {
        UiRebindRecoveryOutcome::Recovered(receipt) => receipt,
        _ => panic!("exact current-frame re-presentation must recover host truth"),
    };
    assert!(receipt.predecessor_remains_current());
    assert_eq!(receipt.mounted().frame(), predecessor.frame());
    assert_eq!(receipt.mounted().generation(), predecessor.generation());
    drop(receipt);
    world.close();
}

#[test]
fn recovery_rejection_retains_the_exact_retry_progression() {
    let mut world = RebindExecutionWorld::new("phase-312-rebind-recovery-retry");
    let host = world.host.clone();
    let mut reconciliation = indeterminate_reconciliation(&mut world);
    let request = rebound_request(&mut reconciliation);
    host.push_rejected();

    let denial = match reconciliation.present_current(request, 2) {
        UiRebindRecoveryOutcome::RejectedBeforeEffects(denial) => denial,
        _ => panic!("pre-effect recovery rejection must retain retry authority"),
    };
    assert_eq!(
        denial.cause(),
        UiRebindRecoveryDenialCause::HostRejectedBeforeEffects
    );
    host.push_presented();
    let receipt = match denial.retry(3) {
        UiRebindRecoveryOutcome::Recovered(receipt) => receipt,
        _ => panic!("the exact retained recovery request must retry"),
    };
    drop(receipt);
    world.close();
}

#[test]
fn recovery_completion_and_reindeterminacy_remain_managed() {
    recovery_completes_from_in_flight();
    repeated_uncertainty_returns_fresh_recovery_authority();
}

#[test]
fn completion_disposal_and_drop_cancel_the_inherited_attempt() {
    completion_disposal_is_typed();
    completion_drop_is_shutdown_clean(UiHostSurfaceCancellationOutcome::CancelledBeforeEffects);
    completion_drop_is_shutdown_clean(UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun);
}

#[test]
fn recovery_completion_disposal_retains_the_recovery_request() {
    let mut world = RebindExecutionWorld::new("phase-312-recovery-completion-disposal");
    let host = world.host.clone();
    let mut reconciliation = indeterminate_reconciliation(&mut world);
    let request = rebound_request(&mut reconciliation);
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let completion = match reconciliation.present_current(request, 2) {
        UiRebindRecoveryOutcome::InFlight(completion) => completion,
        _ => panic!("recovery disposal fixture must begin in flight"),
    };
    let denial = match completion.dispose() {
        UiRebindRecoveryOutcome::RejectedBeforeEffects(denial) => denial,
        _ => panic!("before-effect cancellation must retain recovery retry"),
    };
    assert_eq!(
        denial.cause(),
        UiRebindRecoveryDenialCause::HostRejectedBeforeEffects
    );
    host.push_presented();
    let receipt = match denial.retry(3) {
        UiRebindRecoveryOutcome::Recovered(receipt) => receipt,
        _ => panic!("disposed recovery completion must retain its exact request"),
    };
    drop(receipt);
    world.close();
}

#[test]
fn dropped_recovery_completion_cancels_before_shutdown() {
    let mut world = RebindExecutionWorld::new("phase-312-recovery-completion-drop");
    let host = world.host.clone();
    let mut reconciliation = indeterminate_reconciliation(&mut world);
    let request = rebound_request(&mut reconciliation);
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    let completion = match reconciliation.present_current(request, 2) {
        UiRebindRecoveryOutcome::InFlight(completion) => completion,
        _ => panic!("recovery drop fixture must begin in flight"),
    };
    drop(completion);
    world.close();
}

fn completion_disposal_is_typed() {
    let mut world = RebindExecutionWorld::new("phase-312-rebind-completion-disposal");
    world.host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let prepared = world.prepare_changed();
    let completion = match prepared.execute(1) {
        UiRebindOutcome::InFlight(completion) => completion,
        _ => panic!("completion disposal fixture must begin in flight"),
    };
    let cancellation = match completion.dispose() {
        UiRebindOutcome::CancelledBeforeEffects(cancellation) => cancellation,
        _ => panic!("before-effect cancellation must be terminal and typed"),
    };
    assert!(cancellation.predecessor_remains_current());
    world.close();
}

fn completion_drop_is_shutdown_clean(cancellation: UiHostSurfaceCancellationOutcome) {
    let label = match cancellation {
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects => {
            "phase-312-rebind-completion-drop-before"
        }
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun => {
            "phase-312-rebind-completion-drop-after"
        }
    };
    let mut world = RebindExecutionWorld::new(label);
    world
        .host
        .push_in_flight(vec![ScriptedSurfaceCompletion::Pending], cancellation);
    let prepared = world.prepare_changed();
    let completion = match prepared.execute(1) {
        UiRebindOutcome::InFlight(completion) => completion,
        _ => panic!("completion drop fixture must begin in flight"),
    };
    drop(completion);
    world.close();
}

fn recovery_completes_from_in_flight() {
    let mut world = RebindExecutionWorld::new("phase-312-rebind-recovery-completion");
    let host = world.host.clone();
    let mut reconciliation = indeterminate_reconciliation(&mut world);
    let request = rebound_request(&mut reconciliation);
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending, presented_completion()],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    let completion = match reconciliation.present_current(request, 2) {
        UiRebindRecoveryOutcome::InFlight(completion) => completion,
        _ => panic!("pending reconciliation must remain managed"),
    };
    let completion = match completion.complete(3) {
        UiRebindRecoveryOutcome::InFlight(completion) => completion,
        _ => panic!("pending recovery completion remains managed"),
    };
    let receipt = match completion.complete(4) {
        UiRebindRecoveryOutcome::Recovered(receipt) => receipt,
        _ => panic!("presented recovery completion must reconcile"),
    };
    drop(receipt);
    world.close();
}

fn repeated_uncertainty_returns_fresh_recovery_authority() {
    let mut world = RebindExecutionWorld::new("phase-312-rebind-recovery-indeterminate");
    let host = world.host.clone();
    let mut reconciliation = indeterminate_reconciliation(&mut world);
    let request = rebound_request(&mut reconciliation);
    host.push_presentation(
        worth_ui_runtime::certification_support::ScriptedPresentationOutcome::
            PresentationIndeterminate,
    );
    let recovery = match reconciliation.present_current(request, 2) {
        UiRebindRecoveryOutcome::Indeterminate(recovery) => recovery,
        _ => panic!("uncertain reconciliation must return fresh recovery authority"),
    };
    assert!(!recovery.frame().report().affected_bindings().is_empty());
    let session = recovery.into_session_for_shutdown();
    let _ = session;
    world.close();
}

fn indeterminate_reconciliation(world: &mut RebindExecutionWorld) -> UiRebindReconciliation<'_> {
    world.host.push_presentation(
        worth_ui_runtime::certification_support::ScriptedPresentationOutcome::
            PresentationIndeterminate,
    );
    let prepared = world.prepare_changed();
    match prepared.execute(1) {
        UiRebindOutcome::Indeterminate(recovery) => recovery.begin_reconciliation(),
        _ => panic!("fixture must begin from indeterminate rebind effects"),
    }
}

fn rebound_request(
    reconciliation: &mut UiRebindReconciliation<'_>,
) -> UiRebindReconciliationRequest {
    let affected = reconciliation.affected_bindings()[0];
    let replacement = reconciliation
        .rebind_surface(
            affected,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(2),
        )
        .expect("affected quarantined surface rebinds to known-empty truth");
    UiRebindReconciliationRequest::new(
        vec![UiMountedSurfaceReconciliationBinding::new(
            affected,
            replacement.binding_generation(),
        )]
        .into_boxed_slice(),
        UiPresentationDeadline::at_tick(20),
    )
}
