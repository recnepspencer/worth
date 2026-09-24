use worth_ui::facade::rebind::{
    UiRebindDenialCause, UiRebindDisposition, UiRebindOutcome, UiRebindStoppedPhase,
    UiRebindValidNextAction,
};
use worth_ui_runtime::facade::mounted::UiHostSurfaceCancellationOutcome;

use super::support::RebindExecutionWorld;
use crate::mounted_host_protocol::scripted_host::{
    presented_completion, ScriptedSurfaceCompletion,
};

#[test]
fn changed_rebind_uses_host_effects_and_one_atomic_publication() {
    let mut world = RebindExecutionWorld::new("phase-312-rebind-published");
    let prior = world.session.generation_identity().clone();
    let predecessor_frame = world
        .session
        .current_mounted_publication()
        .expect("fixture publishes a predecessor")
        .frame();
    world.host.push_presented();
    let prepared = world.prepare_changed();
    let candidate = prepared.candidate_generation().clone();

    let receipt = match prepared.execute(1) {
        UiRebindOutcome::Published(receipt) => receipt,
        _ => panic!("presented changed rebind must publish"),
    };
    assert_eq!(receipt.disposition(), UiRebindDisposition::Complete);
    assert_eq!(receipt.prior_generation(), &prior);
    assert_eq!(receipt.active_generation(), &candidate);
    assert!(!receipt.planned_effects().is_empty());
    assert_eq!(
        receipt.planned_cost().effects(),
        receipt.planned_effects().len()
    );
    assert!(!receipt.realized_bindings().is_empty());
    let realized_cost = receipt
        .realized_mount_cost()
        .expect("changed publication carries inherited physical cost");
    assert_eq!(
        realized_cost.adapter().presented_surfaces(),
        receipt.realized_bindings().len() as u64
    );
    assert!(realized_cost.named().retained() > 0);
    assert!(receipt.retains_terminal_decision_record());
    assert!(!receipt.retains_recovery_authority());
    let mounted = receipt
        .mounted_publication()
        .expect("changed publication owns mounted evidence");
    assert_eq!(mounted.predecessor(), Some(predecessor_frame));
    assert_eq!(mounted.generation(), &candidate);
    assert_eq!(world.session.generation_identity(), &candidate);
    assert_eq!(world.session.current_mounted_publication(), Some(mounted));
    drop(receipt);
    world.close();
}

#[test]
fn pre_effect_host_rejection_returns_the_exact_retry_authority() {
    let mut world = RebindExecutionWorld::new("phase-312-rebind-retry");
    let prior = world.session.generation_identity().clone();
    let host = world.host.clone();
    host.push_rejected();
    let prepared = world.prepare_changed();

    let denial = match prepared.execute(1) {
        UiRebindOutcome::RejectedBeforeEffects(denial) => denial,
        _ => panic!("scripted pre-effect rejection must return a denial"),
    };
    assert!(denial.predecessor_remains_current());
    assert_eq!(
        denial.stopped_phase(),
        UiRebindStoppedPhase::HostPresentation
    );
    assert_eq!(
        denial.cause(),
        UiRebindDenialCause::HostRejectedBeforeEffects
    );
    assert_eq!(
        denial.valid_next_action(),
        UiRebindValidNextAction::RetryPrepared
    );
    host.push_presented();
    let receipt = match denial.retry_at(2) {
        UiRebindOutcome::Published(receipt) => receipt,
        _ => panic!("the returned exact retry must publish when the host accepts"),
    };
    assert_eq!(receipt.prior_generation(), &prior);
    drop(receipt);
    world.close();
}

#[test]
fn pending_and_uncertain_effects_never_publish_early() {
    let mut pending = RebindExecutionWorld::new("phase-312-effect-pending");
    let predecessor = pending.session.generation_identity().clone();
    pending.host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending, presented_completion()],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    let prepared = pending.prepare_changed();
    let candidate = prepared.candidate_generation().clone();
    let completion = match prepared.execute(1) {
        UiRebindOutcome::InFlight(completion) => completion,
        _ => panic!("pending host effects must retain completion authority"),
    };
    let completion = match completion.complete(2) {
        UiRebindOutcome::InFlight(completion) => completion,
        _ => panic!("pending host effects remain managed"),
    };
    let receipt = match completion.complete(3) {
        UiRebindOutcome::Published(receipt) => receipt,
        _ => panic!("completed host effects publish exactly once"),
    };
    assert_eq!(receipt.prior_generation(), &predecessor);
    assert_eq!(receipt.active_generation(), &candidate);
    drop(receipt);
    pending.close();

    let mut uncertain = RebindExecutionWorld::new("phase-312-effect-uncertain");
    let predecessor = uncertain.session.generation_identity().clone();
    uncertain.host.push_presentation(
        worth_ui_runtime::certification_support::ScriptedPresentationOutcome::
            PresentationIndeterminate,
    );
    let recovery = match uncertain.prepare_changed().execute(1) {
        UiRebindOutcome::Indeterminate(recovery) => recovery,
        _ => panic!("uncertain effects must retain recovery authority"),
    };
    let session = recovery.into_session_for_shutdown();
    assert_eq!(session.generation_identity(), &predecessor);
    let _ = session;
    uncertain.close();
}
