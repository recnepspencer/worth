use worth_ui::facade::{
    intent::{
        UiIntentConsequencePublicationOutcome, UiIntentDefinition,
        UiIntentExecutionDispatchOutcome, UiIntentRuntimeServiceDestination,
    },
    rebind::{UiRebindExecutionPolicy, UiRebindExecutionRequest},
};
use worth_ui_host_headless::UiHeadlessRecorderCapacity;
use worth_ui_test_support::{
    WorthUiMotionPresentationCertificationExt, WorthUiMountedPublicationCertificationExt,
};

use super::only_transition;
use crate::intent::{
    admission::phase3::world::AdmissionWorld,
    operability::{build_open_portal_application, PrimaryIntent},
};

#[path = "motion_sampling/native_reduced_motion.rs"]
mod native_reduced_motion;
#[path = "motion_sampling/native_sample_replay.rs"]
mod native_sample_replay;
#[path = "motion_sampling/portal_child_commands.rs"]
mod portal_child_commands;
#[path = "motion_sampling/world.rs"]
mod world;
pub(super) use world::{
    assert_motion_tick_applied, launch_scripted_motion_world, motion_tick_batch,
    scripted_motion_host,
};

#[test]
fn before_effect_sample_rejection_discards_the_staged_tick() {
    let host = scripted_motion_host();
    host.push_presented();
    host.push_presented();
    host.push_rejected();
    let mut world = launch_scripted_motion_world(host.clone());
    let installed = world
        .session
        .inspect_motion_presentation_for_certification();
    let presentation = installed.presentation().expect("Motion is installed");

    assert_motion_tick_applied(
        world
            .session
            .admit_host_interaction_batch(motion_tick_batch(&world.session, presentation, 3, 1)),
    );

    let discarded = world
        .session
        .inspect_motion_presentation_for_certification();
    assert_eq!(discarded.last_tick(), None);
    assert_eq!(discarded.presentation(), Some(presentation));
    assert_eq!(discarded.semantic_publications(), 1);
    assert_eq!(host.presentation_calls(), 3);
}

#[test]
fn in_flight_sample_commits_only_after_host_completion() {
    use worth_ui_host_contract::UiHostPresentationEpoch;
    use worth_ui_runtime::certification_support::ScriptedSurfaceCompletion;
    use worth_ui_runtime::facade::mounted::{
        UiHostPresentationCostInput, UiHostPresentationCostReport,
        UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationMode, UiMountedCompletedEffects,
        UiMountedEffectFamily, UiMountedSurfacePresentationCompletion,
    };

    let host = scripted_motion_host();
    host.push_presented();
    host.push_presented();
    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Presented(
            UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::RecordOnly,
                UiHostPresentationEpoch::issued_by_host(2),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::RecordedProjection]),
                UiHostPresentationCostReport::from_adapter(UiHostPresentationCostInput {
                    presented_surfaces: 1,
                    ..Default::default()
                }),
            ),
        )],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let mut world = launch_scripted_motion_world(host);
    let presentation = world
        .session
        .inspect_motion_presentation_for_certification()
        .presentation()
        .expect("Motion is installed");

    assert_motion_tick_applied(
        world
            .session
            .admit_host_interaction_batch(motion_tick_batch(&world.session, presentation, 3, 1)),
    );
    assert_eq!(
        world
            .session
            .inspect_motion_presentation_for_certification()
            .last_tick(),
        None
    );

    world.session.complete_motion_sample_for_certification();
    let completed = world
        .session
        .inspect_motion_presentation_for_certification();
    assert_eq!(completed.last_tick(), Some(1));
    assert_eq!(
        completed.presentation().map(|basis| basis.epoch()),
        Some(UiHostPresentationEpoch::issued_by_host(2))
    );
}

#[test]
fn indeterminate_sample_suspends_readiness_hit_testing_and_sampling_until_reconstruction() {
    use worth_ui::facade::app::{UiMountedFrameOutcome, UiPresentationDeadline};

    let host = scripted_motion_host();
    host.push_presented();
    host.push_presented();
    host.push_presentation(
        worth_ui_runtime::facade::mounted::UiHostSurfacePresentationOutcome::
            PresentationIndeterminate,
    );
    let mut world = launch_scripted_motion_world(host.clone());
    let installed = world
        .session
        .inspect_motion_presentation_for_certification();
    let presentation = installed.presentation().expect("Motion is installed");

    assert_motion_tick_applied(
        world
            .session
            .admit_host_interaction_batch(motion_tick_batch(&world.session, presentation, 3, 1)),
    );
    let unavailable = world
        .session
        .inspect_motion_presentation_for_certification();
    assert_eq!(unavailable.last_tick(), None);
    assert_eq!(unavailable.active_tracks(), 1);
    assert!(!unavailable.sampling_ready());
    assert!(!unavailable.hit_test_truth_available());

    let presentation_calls_before_suppressed_tick = host.presentation_calls();
    assert_motion_tick_applied(
        world
            .session
            .admit_host_interaction_batch(motion_tick_batch(&world.session, presentation, 4, 2)),
    );
    let denied = world
        .session
        .inspect_motion_presentation_for_certification();
    assert_eq!(denied.last_tick(), None);
    assert_eq!(
        denied.sampling_denials(),
        unavailable.sampling_denials(),
        "unavailable presentation truth suppresses sampling before preparation"
    );
    assert_eq!(
        host.presentation_calls(),
        presentation_calls_before_suppressed_tick,
        "suppressed ticks must not reach the presentation host"
    );

    host.push_presented();
    let prepared = crate::filesystem_mounted_world::prepare_frame(&mut world.session)
        .expect("current mounted truth prepares owner reconstruction");
    let publication = match world.session.present_prepared_mounted_frame(
        prepared,
        UiPresentationDeadline::at_tick(1_000),
        3,
    ) {
        UiMountedFrameOutcome::Published(receipt)
        | UiMountedFrameOutcome::Unchanged(receipt)
        | UiMountedFrameOutcome::Reconciled(receipt) => receipt,
        _ => panic!("ordinary mounted publication must reconstruct unavailable sample truth"),
    };
    let recovered_epoch =
        crate::mounted_application_lifecycle::published_mounted_world::presented_epoch(
            &world.session,
            publication.frame(),
            presentation.binding(),
        );
    let recovered_presentation =
        worth_ui::facade::observation_report::UiHostObservationPresentationBasis::new(
            presentation.host_surface(),
            publication.frame(),
            presentation.binding(),
            recovered_epoch,
        );
    let recovered = world
        .session
        .inspect_motion_presentation_for_certification();
    assert!(recovered.sampling_ready());
    assert!(recovered.hit_test_truth_available());

    host.push_presented();
    assert_motion_tick_applied(
        world
            .session
            .admit_host_interaction_batch(motion_tick_batch(
                &world.session,
                recovered_presentation,
                5,
                3,
            )),
    );
    assert_eq!(
        world
            .session
            .inspect_motion_presentation_for_certification()
            .last_tick(),
        Some(3)
    );
}

#[test]
fn validated_tick_observations_sample_motion_without_new_semantic_frames() {
    let (application, facts, recorder) =
        build_open_portal_application(UiHeadlessRecorderCapacity::new(8, 8, 16_384));
    let semantic_text = worth_ui_runtime::facade::entry::UiNativeComponentSemanticTextChange::new(
        "component:visual.identity.component.hit_only",
        "Portal motion content",
    )
    .expect("the authored Portal child accepts semantic text");
    let mut world = AdmissionWorld::launch_application_with_target_and_semantic_text(
        application,
        facts,
        1,
        2,
        [18, 20],
        &[semantic_text],
    );
    let definition = UiIntentDefinition::<PrimaryIntent>::runtime_service(
        UiIntentRuntimeServiceDestination::OpenPortal,
    );
    let admitted = world.admit_exact_definition(0, definition);
    assert!(matches!(
        world
            .session
            .dispatch_admitted_intent(admitted, super::super::execution_deadline(20),),
        UiIntentExecutionDispatchOutcome::AttemptPrepared(_)
    ));
    let handle = only_transition(&mut world)
        .into_consequence()
        .expect("completed portal intent retains its mounted consequence");
    match world.session.publish_intent_consequences(
        handle,
        UiRebindExecutionPolicy::ordinary(),
        UiRebindExecutionRequest::new(40),
    ) {
        UiIntentConsequencePublicationOutcome::Published(_) => {}
        UiIntentConsequencePublicationOutcome::Stopped(stop) => {
            panic!("authored Portal publication stopped: {:?}", stop.reason())
        }
        _ => panic!("authored Portal publication did not settle synchronously"),
    }

    let installed = world
        .session
        .inspect_motion_presentation_for_certification();
    assert_eq!(installed.active_tracks(), 1);
    assert_eq!(installed.retained_samples(), 1);
    assert_eq!(installed.semantic_publications(), 1);
    let presentation = installed
        .presentation()
        .expect("installed sample has a basis");
    let open_transcripts = recorder.observed_transcripts();
    let open_transcript = open_transcripts
        .last()
        .expect("Portal publication records its authored presentation");
    let portal = open_transcript.portal_overlays()[0];
    let trigger = portal.owner();
    let portal_child = open_transcript
        .filled_rects()
        .iter()
        .find(|fill| {
            fill.mounted_instance() != trigger
                && fill.layer_semantic_order() > portal.layer_semantic_order()
        })
        .expect("the real Portal fixture publishes its authored child above the overlay")
        .mounted_instance();
    assert_ne!(portal_child, trigger);
    let portal_child_commands =
        portal_child_commands::exact_portal_child_commands(open_transcript, portal_child, portal);
    let mut portal_motion_commands = portal_child_commands;
    portal_motion_commands
        .insert(worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay(&portal));
    let semantic_transcript_count = recorder.observed_transcripts().len();
    let first_tick = motion_tick_batch(&world.session, presentation, 3, 1);
    assert_motion_tick_applied(world.session.admit_host_interaction_batch(first_tick));
    let active = world
        .session
        .inspect_motion_presentation_for_certification();
    assert_eq!(active.last_tick(), Some(1));
    assert_eq!(active.active_tracks(), 1);
    assert_eq!(active.semantic_publications(), 1);
    assert_ne!(active.presentation(), Some(presentation));
    let active_presentation = active
        .presentation()
        .expect("host sample advances the epoch");
    assert_eq!(active_presentation.frame(), presentation.frame());
    let retained_sample = recorder
        .retained_sample_observation(active_presentation.binding())
        .expect("headless retained presentation exposes committed sample truth");
    assert_eq!(retained_sample.frame(), presentation.frame());
    assert_eq!(retained_sample.epoch(), active_presentation.epoch());
    assert!(!retained_sample.changes().is_empty());
    assert_eq!(
        retained_sample
            .changes()
            .iter()
            .map(|change| change.command())
            .collect::<std::collections::HashSet<_>>(),
        portal_motion_commands,
        "the sample must move the overlay and every emitted child command, but no ordinary trigger command"
    );
    assert!(retained_sample.changes().iter().any(|change| {
        let Some(transform) = change.transform() else {
            return false;
        };
        transform.sampled().y() == transform.source().y() + 8.0 && change.opacity().factor() == 0.0
    }));
    native_sample_replay::assert_first_entrance_sample_clears_published_successor(
        portal,
        &retained_sample,
    );
    assert_eq!(
        recorder.observed_transcripts().len(),
        semantic_transcript_count
    );

    let mid_tick = motion_tick_batch(&world.session, active_presentation, 4, 71);
    assert_motion_tick_applied(world.session.admit_host_interaction_batch(mid_tick));
    let mid = world
        .session
        .inspect_motion_presentation_for_certification();
    let mid_presentation = mid
        .presentation()
        .expect("mid-track sample advances the host epoch");
    assert_eq!(mid.last_tick(), Some(71));
    assert_eq!(mid_presentation.frame(), presentation.frame());
    let retained_mid = recorder
        .retained_sample_observation(mid_presentation.binding())
        .expect("headless host retains the mid-track override");
    assert_eq!(retained_mid.epoch(), mid_presentation.epoch());
    assert_eq!(
        retained_mid
            .changes()
            .iter()
            .map(|change| change.command())
            .collect::<std::collections::HashSet<_>>(),
        portal_motion_commands
    );
    assert!(retained_mid.changes().iter().all(|change| {
        let Some(transform) = change.transform() else {
            return false;
        };
        transform.sampled().y() > transform.source().y()
            && transform.sampled().y() < transform.source().y() + 8.0
            && change.opacity().units() == 57_343
            && mid.opacity_units() == Some(57_343)
    }));

    let repeated_tick = motion_tick_batch(&world.session, mid_presentation, 5, 71);
    assert_motion_tick_applied(world.session.admit_host_interaction_batch(repeated_tick));
    let denied = world
        .session
        .inspect_motion_presentation_for_certification();
    assert_eq!(denied.last_tick(), Some(71));
    assert_eq!(denied.sampling_denials(), 1);
    assert!(denied.last_denial_was_non_monotonic());
    assert_eq!(denied.semantic_publications(), 1);

    let terminal_tick = motion_tick_batch(
        &world.session,
        denied
            .presentation()
            .expect("discarded sampling retains the presented basis"),
        6,
        141,
    );
    assert_motion_tick_applied(world.session.admit_host_interaction_batch(terminal_tick));
    let terminal = world
        .session
        .inspect_motion_presentation_for_certification();
    assert_eq!(terminal.active_tracks(), 0);
    assert_eq!(terminal.retained_samples(), 1);
    assert_eq!(terminal.semantic_publications(), 2);
    assert_eq!(terminal.last_tick(), Some(141));
    assert_eq!(terminal.opacity_units(), Some(u16::MAX));
    assert_eq!(
        recorder.observed_transcripts().len(),
        semantic_transcript_count
    );
}
