use worth_ui::facade::app::{
    WorthUiNativeManagedRebindStop, WorthUiNativeManagedSourceRebindOutcome,
};
use worth_ui::facade::rebind::{
    UiRebindExecutionRequest, UiRebindOutcome, UiRebindPreparationDenial, UiSourceRebindRequest,
};
use worth_ui::facade::source::{
    WorthUiSettledSourceSnapshot, WorthUiSourceEventIngress, WorthUiSourceProvider,
    WorthUiWatcherEvent,
};
use worth_ui_certification::scenario::filesystem_application_lifecycle::FilesystemApplicationLifecycleScenario;

use super::support::RebindExecutionWorld;
use crate::mounted_host_protocol::scripted_host::ScriptedPresentationHost;

#[test]
fn publication_makes_an_older_plan_stale_before_host_effects() {
    let mut world = RebindExecutionWorld::new("phase-312-tt01-stale-plan");
    let stale = world.changed_plan();
    let evidence_only = world.evidence_only_plan();
    let predecessor = world.session.generation_identity().clone();
    let presentation_calls = world.host.presentation_calls();

    let prepared = world
        .session
        .prepare_rebind(evidence_only, UiRebindExecutionRequest::new(1))
        .expect("current evidence-only plan prepares");
    let published = match prepared.execute(1) {
        UiRebindOutcome::Published(receipt) => receipt,
        _ => panic!("evidence-only plan must publish a fresh authored generation"),
    };
    assert_eq!(published.prior_generation(), &predecessor);
    assert_ne!(published.active_generation(), &predecessor);
    drop(published);
    assert_eq!(world.host.presentation_calls(), presentation_calls);

    assert!(matches!(
        world
            .session
            .prepare_rebind(stale, UiRebindExecutionRequest::new(2)),
        Err(UiRebindPreparationDenial::StalePredecessorGeneration)
    ));
    assert_eq!(world.host.presentation_calls(), presentation_calls);
    world.close();
}

#[test]
fn newer_owner_order_supersedes_historical_source_without_host_effects() {
    let scenario = FilesystemApplicationLifecycleScenario::new("phase-312-tt01-superseded");
    let provider_id = "phase-312-tt01-source-order";
    let initial = snapshot(
        provider_id,
        &FilesystemApplicationLifecycleScenario::current_source_text(),
        1,
    );
    let historical = initial.clone();
    let capabilities = scenario.capability_application();
    let submission = FilesystemApplicationLifecycleScenario::lower_snapshot(
        initial,
        capabilities.capabilities(),
    );
    let host = ScriptedPresentationHost::default();
    let app = scenario.prepare_application_with_host(submission, host.clone());
    let mut shell = app
        .launch_native_surface()
        .expect("ordinary native shell launches");
    let generation = shell.generation_identity().clone();
    let newer = snapshot(
        provider_id,
        &FilesystemApplicationLifecycleScenario::candidate_source_text(),
        2,
    );

    let timed_out = UiSourceRebindRequest::new(newer)
        .with_deadline(shell.rebind_deadline_at(0))
        .observed_at_tick(1);
    assert!(matches!(
        shell
            .begin_source_rebind(timed_out)
            .expect("newer source reaches final admission"),
        WorthUiNativeManagedSourceRebindOutcome::Stopped(
            WorthUiNativeManagedRebindStop::TimedOutBeforeEffects(_)
        )
    ));
    assert!(matches!(
        shell
            .begin_source_rebind(UiSourceRebindRequest::new(historical).observed_at_tick(2))
            .expect("historical source is a terminal posture"),
        WorthUiNativeManagedSourceRebindOutcome::Stopped(
            WorthUiNativeManagedRebindStop::SupersededBeforeEffects(_)
        )
    ));
    assert_eq!(shell.generation_identity(), &generation);
    assert_eq!(host.presentation_calls(), 0);
    assert!(shell.shutdown().host_session_released());
}

fn snapshot(provider_id: &str, source: &str, sequence: u64) -> WorthUiSettledSourceSnapshot {
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
