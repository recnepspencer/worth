use worth_signal::facade::AspectMask;
use worth_ui_host_contract::{UiGlyphRasterDemandIdentity, UiGlyphRasterTransactionPending};

use super::{
    declarations::UiNativePhysicalSignalOperation, UiNativePhysicalSignalOwner,
    UiNativePhysicalSignalSettlement, UiNativePhysicalSignalStatus,
};

mod presentation_timeout;
mod request_locality;

#[test]
pub(super) fn one_runtime_owns_signal_admission_completion_and_quiescent_shutdown() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let runtime_identity = owner.observation().runtime;
    let initial_telemetry = owner.worker.as_ref().unwrap().telemetry();
    let pending = UiGlyphRasterTransactionPending::from_text_mechanics(
        UiGlyphRasterDemandIdentity::from_text_mechanics([31; 32]),
        2,
        7,
        11,
    );

    let token = admit_atlas_upload(&mut owner, pending);
    let admitted_graph = owner.observation();
    assert_eq!(admitted_graph.signal_performed_transitions, 2);
    assert!(admitted_graph.signal_performed_nodes >= 2);
    assert_eq!(owner.observation().runtime, runtime_identity);
    assert_eq!(owner.observation().active_requests, 1);
    assert_eq!(owner.observation().pending_wakes, 0);
    assert_eq!(
        owner.reconcile(token.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Completed
    );

    let completed = owner.worker.as_ref().unwrap().telemetry();
    assert!(
        completed.resource.resource_request_admission_count
            > initial_telemetry.resource.resource_request_admission_count
    );
    assert!(
        completed.resource.resource_completion_commit_count
            > initial_telemetry.resource.resource_completion_commit_count
    );
    assert!(
        initial_telemetry
            .resource
            .async_node_capability_attachment_count
            > 0
    );
    assert_eq!(owner.observation().active_requests, 0);
    assert_eq!(owner.observation().pending_wakes, 0);
    let settled_graph = owner.observation();
    assert!(
        settled_graph.signal_performed_transitions > admitted_graph.signal_performed_transitions
    );
    assert!(settled_graph.signal_performed_nodes > admitted_graph.signal_performed_nodes);
    let [transition] = owner.transition_observations() else {
        panic!("one physical completion must retain one owner-issued transition");
    };
    assert_eq!(
        transition.work(),
        super::UiNativePhysicalSignalWorkClass::AtlasUpload
    );
    assert_eq!(
        transition.external_status(),
        super::UiNativePhysicalSignalExternalStatusClass::Completed
    );
    assert_eq!(
        transition.settlement(),
        super::UiNativePhysicalSignalSettlementClass::Completed
    );
    assert_eq!(transition.performed_transitions(), 1);
    assert!(transition.performed_nodes() > 0);
    assert!(owner.transition_observation_trace_complete());
    assert_eq!(
        owner.shutdown(),
        super::shutdown::UiNativePhysicalSignalShutdown::Disposed
    );
    assert_eq!(
        owner.observation().signal_performed_transitions,
        settled_graph.signal_performed_transitions,
        "disposed Signal retains its performed progression receipt"
    );
    assert!(owner.transition_observations().is_empty());
}

#[test]
fn operation_families_consume_distinct_exact_aspect_masks() {
    let owner = UiNativePhysicalSignalOwner::new();
    let declarations = owner.declarations();
    let masks = declarations.resources.map(|resource| resource.reads());
    assert!(masks.iter().all(|mask| *mask != AspectMask::ALL));
    assert_ne!(
        masks[UiNativePhysicalSignalOperation::AtlasUpload.index()],
        masks[UiNativePhysicalSignalOperation::PresentationReadback.index()]
    );
    assert_ne!(
        masks[UiNativePhysicalSignalOperation::PresentationReadback.index()],
        masks[UiNativePhysicalSignalOperation::Recovery.index()]
    );
}

#[test]
fn partitioned_progression_evaluates_only_the_exact_operation_family() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let initial = owner.observation().signal_performed_nodes;
    let atlas = pending(34);
    let _ = admit_atlas_upload(&mut owner, atlas);
    let after_atlas = owner.observation().signal_performed_nodes;
    assert_eq!(
        after_atlas - initial,
        10,
        "planning and upload each evaluate only the five-source atlas partition"
    );

    let presentation = owner
        .admit_presentation(super::UiNativePhysicalPresentationBasis::test())
        .unwrap();
    let after_presentation = owner.observation().signal_performed_nodes;
    assert_eq!(
        after_presentation - after_atlas,
        4,
        "presentation progression cannot evaluate atlas or recovery nodes"
    );
    let _ = owner.take_initial_presentation(presentation).unwrap();
}

#[test]
pub(super) fn wake_delivery_is_exact_to_the_ready_physical_work() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let atlas = pending(35);
    let atlas_token = admit_atlas_upload(&mut owner, atlas);
    let presentation = owner
        .admit_presentation(super::UiNativePhysicalPresentationBasis::test())
        .unwrap();
    assert_eq!(owner.observation().pending_wakes, 1);

    let presentation_token = owner.take_initial_presentation(presentation).unwrap();
    assert_eq!(owner.observation().pending_wakes, 0);
    assert!(owner.take_initial_presentation(presentation).is_err());

    assert_eq!(
        owner.reconcile(presentation_token.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Completed
    );
    assert_eq!(
        owner.reconcile(atlas_token.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Completed
    );
}

#[test]
pub(super) fn signal_owns_retry_timeout_cancel_supersession_and_retained_shutdown() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let retry = pending(41);
    let _retry_attempt = admit_atlas_upload(&mut owner, retry);
    assert_eq!(owner.next_due_tick(), Some(8));
    owner.advance_clock_to(7).unwrap();
    assert!(
        owner.begin(retry).is_ok(),
        "the request remains current before its deadline"
    );
    owner.advance_clock_to(8).unwrap();
    assert_eq!(owner.observation().pending_wakes, 0);
    assert_eq!(owner.next_due_tick(), Some(9));
    owner.advance_clock_to(9).unwrap();
    let retry_token = owner
        .take_ready_atlas_upload(retry)
        .expect("the due Signal retry replaces the expired attempt");
    assert_eq!(
        owner.reconcile(retry_token.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Completed
    );

    let timed_out = pending(42);
    let _timed_out_attempt = admit_atlas_upload(&mut owner, timed_out);
    owner.advance_clock_to(17).unwrap();
    assert_eq!(owner.observation().pending_wakes, 0);
    owner.advance_clock_to(18).unwrap();
    assert_eq!(owner.observation().pending_wakes, 1);
    owner.take_ready_atlas_upload(timed_out).unwrap();
    assert_eq!(
        owner.cancel_atlas_upload(timed_out),
        UiNativePhysicalSignalSettlement::Rejected
    );

    let cancelled = pending(43);
    admit_atlas_upload(&mut owner, cancelled);
    assert_eq!(
        owner.cancel_atlas_upload(cancelled),
        UiNativePhysicalSignalSettlement::Rejected
    );

    let predecessor = pending(44);
    admit_atlas_upload(&mut owner, predecessor);
    let recovery = owner
        .supersede_atlas_upload_to_recovery(predecessor)
        .expect("Signal supersedes the exact predecessor into recovery");
    assert!(owner.token_uses_recovery(recovery));
    assert_eq!(
        owner.shutdown(),
        super::shutdown::UiNativePhysicalSignalShutdown::RetainedObligations { active_requests: 1 }
    );
    assert_eq!(
        owner.reconcile(recovery.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Completed,
        "shutdown revokes admission but still lets retained recovery complete"
    );
    assert_eq!(
        owner.shutdown(),
        super::shutdown::UiNativePhysicalSignalShutdown::Disposed
    );
}

#[test]
fn exhausted_physical_attempts_become_signal_owned_recovery() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let work = pending(46);
    let initial = admit_atlas_upload(&mut owner, work);
    assert_eq!(
        owner.reconcile(initial.observe(UiNativePhysicalSignalStatus::Pending)),
        UiNativePhysicalSignalSettlement::Pending
    );

    for _ in 0..40 {
        if owner.observation().counters.recovery_schedules != 0 {
            break;
        }
        let due = owner
            .next_due_tick()
            .expect("bounded work retains a due transition");
        owner.advance_clock_to(due).unwrap_or_else(|()| {
            panic!(
                "physical transition failed at tick {due}: {:?}",
                owner.observation()
            )
        });
        if owner.observation().counters.recovery_schedules != 0 {
            break;
        }
        if matches!(
            owner.next_ready_work(),
            Some(super::UiNativePhysicalSignalWork::AtlasUpload(identity))
                if identity.pending() == work
        ) {
            let token = owner.take_ready_atlas_upload(work).unwrap();
            let _ = owner.reconcile(token.observe(UiNativePhysicalSignalStatus::Pending));
        }
    }

    let observation = owner.observation();
    assert_eq!(observation.counters.recovery_schedules, 1);
    assert_eq!(observation.active_requests, 1);
    assert_eq!(observation.pending_wakes, 1);
}

#[test]
pub(super) fn effects_indeterminate_transitions_to_signal_owned_recovery_until_completion() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let work = pending(51);
    let token = admit_atlas_upload(&mut owner, work);
    assert_eq!(
        owner.reconcile(token.observe(UiNativePhysicalSignalStatus::EffectsIndeterminate)),
        UiNativePhysicalSignalSettlement::Indeterminate
    );
    let recovery = owner
        .transition_atlas_upload_to_recovery(work)
        .expect("Signal replaces the upload request with exact recovery work");
    assert_eq!(owner.observation().active_requests, 1);
    assert_eq!(
        owner.observation().recovery,
        super::UiNativePhysicalRecoveryPosture::Required { active_requests: 1 }
    );
    owner.take_ready_token(recovery).unwrap();
    assert_eq!(
        owner.reconcile(recovery.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Completed
    );
    assert_eq!(owner.observation().active_requests, 0);
    assert_eq!(
        owner.observation().recovery,
        super::UiNativePhysicalRecoveryPosture::Resolved {
            total_resolutions: 1
        }
    );
}

#[test]
pub(super) fn before_effect_rejection_is_a_terminal_signal_observation() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let work = pending(52);
    let token = admit_atlas_upload(&mut owner, work);
    assert_eq!(
        owner.reconcile(token.observe(UiNativePhysicalSignalStatus::RejectedBeforeEffects)),
        UiNativePhysicalSignalSettlement::Rejected
    );
    assert_eq!(owner.observation().active_requests, 0);
}

#[test]
pub(super) fn foreign_duplicate_and_out_of_order_completion_envelopes_are_stale() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let mut foreign = UiNativePhysicalSignalOwner::new();
    let work = pending(53);
    let foreign_work = pending(54);
    let token = admit_atlas_upload(&mut owner, work);
    let foreign_token = admit_atlas_upload(&mut foreign, foreign_work);

    assert_eq!(
        owner.reconcile(foreign_token.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Stale
    );
    let completed = token.observe(UiNativePhysicalSignalStatus::Completed);
    assert_eq!(
        owner.reconcile(completed),
        UiNativePhysicalSignalSettlement::Completed
    );
    assert_eq!(
        owner.reconcile(completed),
        UiNativePhysicalSignalSettlement::Stale
    );
}

#[test]
fn retained_host_work_settles_as_superseded_before_the_current_presentation() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let predecessor_basis = super::UiNativePhysicalPresentationBasis::test();
    let predecessor = owner.admit_presentation(predecessor_basis).unwrap();
    let predecessor_token = owner.take_initial_presentation(predecessor).unwrap();
    let current = owner
        .admit_presentation(predecessor_basis.test_successor())
        .unwrap();
    let current_token = owner.take_initial_presentation(current).unwrap();

    assert_eq!(
        owner.reconcile(predecessor_token.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Superseded
    );
    assert_eq!(owner.observation().active_requests, 1);
    assert_eq!(
        owner.reconcile(current_token.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Completed
    );
    assert_eq!(owner.observation().active_requests, 0);
    assert_eq!(
        owner.transition_observations()[0].settlement(),
        super::UiNativePhysicalSignalSettlementClass::Superseded
    );
}

#[test]
fn unrelated_presentations_keep_independent_physical_signal_currentness() {
    let mut owner = UiNativePhysicalSignalOwner::new();
    let first = owner
        .admit_presentation(super::UiNativePhysicalPresentationBasis::test())
        .unwrap();
    let first_token = owner.take_initial_presentation(first).unwrap();
    let second = owner
        .admit_presentation(super::UiNativePhysicalPresentationBasis::test())
        .unwrap();
    let second_token = owner.take_initial_presentation(second).unwrap();

    assert_eq!(
        owner.reconcile(first_token.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Completed
    );
    assert_eq!(
        owner.reconcile(second_token.observe(UiNativePhysicalSignalStatus::Completed)),
        UiNativePhysicalSignalSettlement::Completed
    );
    assert_eq!(owner.observation().active_requests, 0);
}

fn pending(seed: u8) -> UiGlyphRasterTransactionPending {
    UiGlyphRasterTransactionPending::from_text_mechanics(
        UiGlyphRasterDemandIdentity::from_text_mechanics([seed; 32]),
        u64::from(seed),
        u64::from(seed) + 1,
        u64::from(seed) + 2,
    )
}

fn admit_atlas_upload(
    owner: &mut UiNativePhysicalSignalOwner,
    pending: UiGlyphRasterTransactionPending,
) -> super::UiNativePhysicalSignalRequestToken {
    let pins =
        worth_ui_host_contract::UiGlyphRasterPinTransitionView::from_text_mechanics(&[], &[]);
    let planning = owner
        .admit_atlas_planning(super::UiNativePhysicalPresentationBasis::test(), &[], pins)
        .expect("the bounded physical Signal owner admits atlas planning");
    let token = owner
        .take_ready_atlas_planning(planning)
        .expect("atlas planning emits one ready wake");
    owner
        .bind_atlas_upload(token, pending)
        .expect("the exact atlas request transitions into upload work")
}
