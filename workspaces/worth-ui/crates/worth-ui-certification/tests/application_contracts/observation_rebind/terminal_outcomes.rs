use worth_ui::facade::app::{
    WorthUiNativeApplicationShell, WorthUiNativeManagedRebindStop,
    WorthUiNativeManagedSourceRebindOutcome,
};
use worth_ui::facade::rebind::{
    UiRebindStoppedPhase, UiRebindValidNextAction, UiSourceRebindRequest,
};
use worth_ui::facade::source::{
    WorthUiSettledSourceSnapshot, WorthUiSourceEventIngress, WorthUiSourceProvider,
    WorthUiWatcherEvent,
};
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;

use crate::mounted_host_protocol::scripted_host::ScriptedPresentationHost;

struct NativeSourceRebindWorld {
    _scenario: FilesystemApplicationLifecycleScenario,
    provider_id: String,
    host: ScriptedPresentationHost,
    shell: WorthUiNativeApplicationShell,
}

pub(crate) fn prove_terminal_outcome_cleanup() {
    ordinary_request_distinguishes_no_change_duplicate_and_superseded_evidence();
    ordinary_timeout_stops_at_final_admission_without_host_effects();
    ordinary_cancellation_stops_at_final_admission_without_host_effects();
}

impl NativeSourceRebindWorld {
    fn new(label: &str) -> Self {
        let scenario = FilesystemApplicationLifecycleScenario::new(label);
        let provider_id = format!("phase-312-terminal-{label}");
        let initial = sequenced_snapshot(
            &provider_id,
            &FilesystemApplicationLifecycleScenario::current_source_text(),
            1,
        );
        let capabilities = scenario.capability_application();
        let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
            initial,
            capabilities.capabilities(),
        );
        let host = ScriptedPresentationHost::default();
        let app = scenario.prepare_application_with_host(submission, host.clone());
        let shell = app
            .launch_native_surface()
            .expect("ordinary native shell launches");
        Self {
            _scenario: scenario,
            provider_id,
            host,
            shell,
        }
    }

    fn snapshot(&self, source: &str, sequence: u64) -> WorthUiSettledSourceSnapshot {
        sequenced_snapshot(&self.provider_id, source, sequence)
    }

    fn close(self) {
        let shutdown = self.shell.shutdown();
        assert!(shutdown.host_session_released());
        assert_eq!(shutdown.mounted_shutdown_attempt_count(), 0);
    }
}

#[test]
fn ordinary_request_distinguishes_no_change_duplicate_and_superseded_evidence() {
    let mut world = NativeSourceRebindWorld::new("terminal-order");
    let source = FilesystemApplicationLifecycleScenario::current_source_text();
    let current = world.snapshot(&source, 1);
    let duplicate = current.clone();
    let historical = current.clone();
    let advanced = world.snapshot(
        &FilesystemApplicationLifecycleScenario::candidate_source_text(),
        2,
    );
    let generation = world.shell.generation_identity().clone();

    match world
        .shell
        .begin_source_rebind(UiSourceRebindRequest::new(current).observed_at_tick(1))
        .expect("exact current source reaches classification")
    {
        WorthUiNativeManagedSourceRebindOutcome::Stopped(
            WorthUiNativeManagedRebindStop::ObservedNoChange,
        ) => {}
        outcome => {
            drop(outcome);
            panic!("exact semantic source must be observed no-change");
        }
    }
    assert_eq!(world.shell.generation_identity(), &generation);
    assert_eq!(world.host.presentation_calls(), 0);

    match world
        .shell
        .begin_source_rebind(UiSourceRebindRequest::new(duplicate).observed_at_tick(2))
        .expect("duplicate is a terminal outcome")
    {
        WorthUiNativeManagedSourceRebindOutcome::Stopped(
            WorthUiNativeManagedRebindStop::Duplicate,
        ) => {}
        _ => panic!("equal owner order must be duplicate"),
    }

    let advance = UiSourceRebindRequest::new(advanced)
        .with_deadline(world.shell.rebind_deadline_at(0))
        .observed_at_tick(1);
    match world
        .shell
        .begin_source_rebind(advance)
        .expect("newer owner evidence reaches final admission")
    {
        WorthUiNativeManagedSourceRebindOutcome::Stopped(
            WorthUiNativeManagedRebindStop::TimedOutBeforeEffects(receipt),
        ) => {
            assert!(receipt.predecessor_remains_current());
            assert_eq!(
                receipt.stopped_phase(),
                UiRebindStoppedPhase::FinalAdmission
            );
            assert_eq!(receipt.valid_next_action(), UiRebindValidNextAction::None);
        }
        _ => panic!("newer changed evidence should stop at its expired deadline"),
    }

    match world
        .shell
        .begin_source_rebind(UiSourceRebindRequest::new(historical).observed_at_tick(3))
        .expect("historical evidence is a terminal outcome")
    {
        WorthUiNativeManagedSourceRebindOutcome::Stopped(
            WorthUiNativeManagedRebindStop::SupersededBeforeEffects(receipt),
        ) => {
            assert!(receipt.predecessor_remains_current());
            assert_eq!(
                receipt.stopped_phase(),
                UiRebindStoppedPhase::ObservationAdmission
            );
            assert_eq!(receipt.valid_next_action(), UiRebindValidNextAction::None);
        }
        _ => panic!("lower owner order must be superseded"),
    }
    assert_eq!(world.shell.generation_identity(), &generation);
    assert_eq!(world.host.presentation_calls(), 0);
    world.close();
}

#[test]
fn ordinary_timeout_stops_at_final_admission_without_host_effects() {
    let mut world = NativeSourceRebindWorld::new("terminal-timeout");
    let changed = world.snapshot(
        &FilesystemApplicationLifecycleScenario::candidate_source_text(),
        2,
    );
    let generation = world.shell.generation_identity().clone();
    let request = UiSourceRebindRequest::new(changed)
        .with_deadline(world.shell.rebind_deadline_at(10))
        .observed_at_tick(11);

    match world
        .shell
        .begin_source_rebind(request)
        .expect("elapsed request is a typed terminal outcome")
    {
        WorthUiNativeManagedSourceRebindOutcome::Stopped(
            WorthUiNativeManagedRebindStop::TimedOutBeforeEffects(receipt),
        ) => {
            assert!(receipt.predecessor_remains_current());
            assert_eq!(
                receipt.stopped_phase(),
                UiRebindStoppedPhase::FinalAdmission
            );
            assert_eq!(receipt.valid_next_action(), UiRebindValidNextAction::None);
        }
        _ => panic!("elapsed request must time out before effects"),
    }
    assert_eq!(world.shell.generation_identity(), &generation);
    assert_eq!(world.host.presentation_calls(), 0);
    world.close();
}

#[test]
fn ordinary_cancellation_stops_at_final_admission_without_host_effects() {
    let mut world = NativeSourceRebindWorld::new("terminal-cancellation");
    let changed = world.snapshot(
        &FilesystemApplicationLifecycleScenario::candidate_source_text(),
        2,
    );
    let generation = world.shell.generation_identity().clone();
    let request = UiSourceRebindRequest::new(changed)
        .with_cancellation(world.shell.rebind_cancellation_request())
        .observed_at_tick(1);

    match world
        .shell
        .begin_source_rebind(request)
        .expect("cancelled request is a typed terminal outcome")
    {
        WorthUiNativeManagedSourceRebindOutcome::Stopped(
            WorthUiNativeManagedRebindStop::CancelledBeforeEffects(receipt),
        ) => {
            assert!(receipt.predecessor_remains_current());
            assert_eq!(
                receipt.stopped_phase(),
                UiRebindStoppedPhase::FinalAdmission
            );
            assert_eq!(receipt.valid_next_action(), UiRebindValidNextAction::None);
        }
        _ => panic!("cancelled request must stop before effects"),
    }
    assert_eq!(world.shell.generation_identity(), &generation);
    assert_eq!(world.host.presentation_calls(), 0);
    world.close();
}

fn sequenced_snapshot(
    provider_id: &str,
    source: &str,
    sequence: u64,
) -> WorthUiSettledSourceSnapshot {
    let provider =
        WorthUiSourceProvider::in_memory(provider_id).with_file("app/main.wui", source.to_owned());
    let mut ingress = WorthUiSourceEventIngress::new(provider).start();
    (1..=sequence)
        .map(|_| {
            ingress
                .ingest([WorthUiWatcherEvent::provider_revision(provider_id)])
                .expect("in-memory source settles")
        })
        .last()
        .expect("positive sequence yields one snapshot")
}
