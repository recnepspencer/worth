use worth_ui::facade::app::{
    WorthUiMountedApplicationReplacementOutcome, WorthUiMountedReplacementPreparationOutcome,
};
use worth_ui::facade::intent::UiIntentConsequencePublicationOutcome;
use worth_ui::facade::source::{
    WorthUiSourceIngressExt, WorthUiSourceProvider, WorthUiWatcherEvent,
};
use worth_ui_certification::scenario::application_authority_closure::candidate_catalog::admit_candidate_catalog;
use worth_ui_host_headless::{UiHeadlessRecorderCapacity, WorthUiHeadlessRecorder};
use worth_ui_query_binding::UiLiveCollectionProjectionCloseOutcome;
use worth_ui_runtime::facade::measurement_exchange::UiViewportExtentObservation;
use worth_ui_runtime::facade::mounted::{UiMountedFrameRequest, UiPresentationDeadline};
use worth_ui_test_support::{
    WorthUiFocusRuntimeCertificationExt, WorthUiFrameworkTurnCertificationExt,
    WorthUiMountedIdentityCertificationExt, WorthUiServiceStateCertificationExt,
};

use super::super::world::routed_scroll_selection_input;
use super::focus_reveal_frame::publish_focus_reveal_frame;
use super::scrolled_selection_portal::{
    activate_scrolled_row, dispatch_selection_portal, launch_selection_portal,
    selection_declaration, SelectionPortalWorld, SELECTED_KEY,
};

#[test]
fn declared_selection_portal_rejects_atomically_then_commits_selection_and_focus_reveal() {
    let recorder = WorthUiHeadlessRecorder::with_viewport_extent(
        UiHeadlessRecorderCapacity::new(8, 1, 16_384),
        UiViewportExtentObservation {
            width: 160.0,
            height: 96.0,
        },
    );
    let SelectionPortalWorld {
        mut world,
        mut query,
        live,
        projection,
        option,
    } = launch_selection_portal(recorder.clone());
    let replacement_input = routed_scroll_selection_input(selection_declaration(&projection), true);
    let scrolled = activate_scrolled_row(&mut world);
    let target = scrolled.activation.target().mounted_instance();
    let consequence = dispatch_selection_portal(&mut world, scrolled.activation, option);
    let selection_before = world
        .interaction
        .session
        .inspect_selection_runtime_for_certification();
    let scroll_before = world
        .interaction
        .session
        .inspect_scroll_runtime_for_certification();
    let focus_before = world
        .interaction
        .session
        .inspect_focus_runtime_for_certification();

    let recovery = match world.interaction.session.publish_intent_consequences(
        consequence,
        worth_ui::facade::rebind::UiRebindExecutionPolicy::ordinary(),
        worth_ui::facade::rebind::UiRebindExecutionRequest::new(315_052),
    ) {
        UiIntentConsequencePublicationOutcome::Stopped(stop) => {
            assert!(
                matches!(stop.reason(), worth_ui::facade::intent::UiIntentConsequenceStopReason::HostRejectedBeforeEffects { rejection_count: 1 }),
                "expected recorder rejection, got {:?}", stop.reason(),
            );
            stop.into_recovery()
        }
        _ => panic!("the full recorder must reject before effects"),
    };
    assert_eq!(
        world
            .interaction
            .session
            .inspect_selection_runtime_for_certification(),
        selection_before
    );
    assert_eq!(
        world
            .interaction
            .session
            .inspect_scroll_runtime_for_certification(),
        scroll_before
    );
    assert_eq!(
        world
            .interaction
            .session
            .inspect_focus_runtime_for_certification(),
        focus_before
    );
    assert_eq!(recorder.drain_transcripts().len(), 1);

    match world.interaction.session.retry_intent_consequences(
        recovery,
        worth_ui::facade::rebind::UiRebindExecutionPolicy::ordinary(),
        worth_ui::facade::rebind::UiRebindExecutionRequest::new(315_053),
    ) {
        UiIntentConsequencePublicationOutcome::Published(_) => {}
        UiIntentConsequencePublicationOutcome::Stopped(stop) => {
            panic!("selection Portal retry stopped: {:?}", stop.reason())
        }
        _ => panic!("selection Portal retry did not publish"),
    }
    let selected = world
        .interaction
        .session
        .inspect_selection_runtime_for_certification();
    assert_eq!((selected.owners(), selected.selected_keys()), (1, 1));
    assert_eq!(selected.available_catalog_owners(), 1);
    assert_eq!((selected.requests(), selected.keys_visited()), (1, 1));
    assert_eq!(selected.catalog_keys_reconciled(), 1);
    assert_eq!(
        selected.selected_application_item_keys(),
        &[core::num::NonZeroU64::new(SELECTED_KEY).unwrap()],
    );
    let focused = world
        .interaction
        .session
        .inspect_focus_runtime_for_certification();
    assert_eq!(focused.revision(), focus_before.revision() + 1);
    assert!(focused.current_participant().is_some());
    let revealed = publish_focus_reveal_frame(&mut world, &recorder, scrolled.scrolled_offset);
    assert_eq!((revealed.owners(), revealed.admitted_requests()), (1, 2));
    assert_eq!(revealed.owners_visited(), 2);
    assert_eq!(
        revealed.owner_geometry()[0].block_offset_subpixels(),
        scrolled.expected_reveal_offset,
        "Nearest reveal settles in owner content space, independent of the predecessor offset"
    );

    let _ = recorder.drain_transcripts();
    publish_mounted_replacement(&mut world, replacement_input);
    let replaced = world
        .interaction
        .session
        .inspect_selection_runtime_for_certification();
    assert_eq!((replaced.owners(), replaced.selected_keys()), (1, 1));
    assert_eq!(
        replaced.available_catalog_owners(),
        1,
        "the published successor installs its reconciled current catalog"
    );
    assert_eq!(
        replaced.selected_application_item_keys(),
        &[core::num::NonZeroU64::new(SELECTED_KEY).unwrap()],
        "production replacement preserves the stable selected application key"
    );

    world.interaction.session.unmount_instance(target).unwrap();
    assert_eq!(
        world
            .interaction
            .session
            .inspect_selection_runtime_for_certification()
            .owners(),
        0,
        "unmount retires the exact Selection owner immediately"
    );
    assert_eq!(
        world
            .interaction
            .session
            .inspect_scroll_runtime_for_certification()
            .owners(),
        0,
        "unmount retires the exact region-owned Scroll state immediately"
    );

    match live.close(&mut query) {
        UiLiveCollectionProjectionCloseOutcome::Closed(closed) => assert!(closed.owner_terminal()),
        UiLiveCollectionProjectionCloseOutcome::Stopped(stop) => {
            panic!(
                "selection portal Query owner closes: {:?}",
                stop.query_error()
            )
        }
    }
    let _ = world.interaction.session.shutdown();
}

fn publish_mounted_replacement(
    world: &mut super::super::world::PayloadWorld,
    input: worth_ui_dsl::WorthUiRustAuthoredArtifactInput,
) {
    const PROVIDER: &str = "phase-315-selection-replacement";
    let provider = WorthUiSourceProvider::rust_authored(PROVIDER).with_rust_authored_input(input);
    let mut ingress = world
        .interaction
        .session
        .source_event_ingress(provider)
        .start();
    let settled = ingress
        .ingest([WorthUiWatcherEvent::provider_revision(PROVIDER)])
        .expect("replacement Rust source settles through production ingress");
    let submission = settled
        .attempt_candidate_for_certification(world.interaction.session.capabilities())
        .expect("selection successor lowers through the production compiler");
    let mut prepared = world
        .interaction
        .session
        .prepare_replacement(submission)
        .expect("selection successor prepares");
    let catalog = admit_candidate_catalog(&world.interaction.session, &mut prepared);
    let lowered = world
        .interaction
        .session
        .lower_prepared_replacement(*prepared)
        .expect("selection successor lowers");
    let pending = world
        .interaction
        .session
        .stage_prepared_replacement(lowered)
        .expect("selection successor stages");
    let boundary = world
        .interaction
        .session
        .execute_framework_turn(|_| {})
        .expect("no mounted presentation lease is active")
        .into_completion()
        .into_execution()
        .unwrap_or_else(|_| panic!("replacement boundary turn completes"))
        .into_activation_boundary();
    let replacement = match world
        .interaction
        .session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .expect("selection successor prepares one mounted replacement")
    {
        WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
        WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
            panic!("changed component requires one mounted successor")
        }
    };
    assert!(matches!(
        replacement.present(UiPresentationDeadline::at_tick(315_054), 1),
        WorthUiMountedApplicationReplacementOutcome::Published { .. }
    ));
}
