use worth_ui::facade::app::{UiMountedFrameOutcome, UiMountedFrameRequest, UiPresentationDeadline};
use worth_ui_host_headless::WorthUiHeadlessRecorder;
use worth_ui_runtime::facade::mounted::{UiHostSurfacePresentationMode, UiMountedFrameReuse};
use worth_ui_test_support::{
    WorthUiFrameworkTurnCertificationExt, WorthUiMountedFrameExecutionCertificationExt,
    WorthUiMountedIdentityCertificationExt,
};

use super::mounted_application_lifecycle::known_empty_surface_world::{
    first_node, mounted_application_with_host, profile,
};

// The changed path retains predecessor protocol state plus receipt, order,
// damage, and Gate 4 appearance-membership indexes. This local ceiling
// complements the retained-scale slope proofs; it is not itself an O(k) claim.
const MAX_CHANGED_PUBLIC_ALLOCATIONS: u64 = 192;
const MAX_CHANGED_PUBLIC_BYTES: u64 = 64 * 1_024;

#[test]
fn empty_public_framework_turn_acquisition_is_allocation_free() {
    let mut session = mounted_application_with_host(
        "phase7-framework-turn-allocation",
        WorthUiHeadlessRecorder::default(),
    )
    .launch()
    .expect("real file-authored application launches");
    drop(
        session
            .execute_framework_turn(|_| {})
            .expect("warm empty framework turn")
            .into_completion(),
    );

    let mut completion = None;
    let allocations = allocation_counter::measure(|| {
        completion = Some(
            session
                .execute_framework_turn(|_| {})
                .expect("measured empty framework turn"),
        );
    });
    drop(
        completion
            .expect("allocator observer captures framework completion")
            .into_completion(),
    );
    assert_eq!(
        allocations.count_total, 0,
        "empty framework-turn acquisition allocated {allocations:?}"
    );
    let _ = session.shutdown();
}

#[test]
fn exact_reuse_classification_is_allocation_free() {
    let mut session = mounted_application_with_host(
        "phase7-reuse-classification-allocation",
        WorthUiHeadlessRecorder::default(),
    )
    .launch()
    .expect("real file-authored application launches");
    mount_one(&mut session);
    let request = UiMountedFrameRequest::all_bound_surfaces();
    let _ = session
        .execute_mounted_frame(
            request.clone(),
            UiPresentationDeadline::at_tick(10),
            0,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("initial public mounted frame executes"));

    let completion = session
        .execute_framework_turn(|_| {})
        .expect("empty framework turn completes");
    let mut execution = None;
    let conversion_allocations = allocation_counter::measure(|| {
        execution = Some(
            completion
                .into_execution()
                .unwrap_or_else(|_| panic!("empty framework turn is executable")),
        );
    });
    assert_eq!(
        conversion_allocations.count_total, 0,
        "framework execution conversion allocated {conversion_allocations:?}"
    );

    let execution = execution.expect("allocation observer captures execution");
    let mut reuse = None;
    let classification_allocations = allocation_counter::measure(|| {
        reuse = Some(execution.classify_mounted_frame_reuse(&request));
    });
    assert_eq!(
        classification_allocations.count_total, 0,
        "exact mounted reuse classification allocated {classification_allocations:?}"
    );
    assert!(matches!(reuse, Some(UiMountedFrameReuse::Exact(_))));
    drop(execution);
    let _ = session.shutdown();
}

#[test]
fn public_unchanged_is_allocation_free_and_one_instance_change_is_bounded() {
    let recorder = WorthUiHeadlessRecorder::default();
    let mut session =
        mounted_application_with_host("phase7-public-mounted-allocation", recorder.clone())
            .launch()
            .expect("real file-authored application launches");
    let surface = mount_one(&mut session);
    let request = UiMountedFrameRequest::all_bound_surfaces();
    let first = published(
        session
            .execute_mounted_frame(
                request.clone(),
                UiPresentationDeadline::at_tick(10),
                0,
                |_| {},
            )
            .unwrap_or_else(|_| panic!("initial public mounted frame executes")),
    );

    let mut unchanged = None;
    let unchanged_allocations = allocation_counter::measure(|| {
        unchanged = Some(
            session
                .execute_mounted_frame(
                    request.clone(),
                    UiPresentationDeadline::at_tick(11),
                    1,
                    |_| {},
                )
                .unwrap_or_else(|_| panic!("unchanged public mounted frame executes")),
        );
    });
    let unchanged = unchanged.expect("allocation observer captures unchanged outcome");
    let unchanged_cost = unchanged
        .cost_report()
        .expect("unchanged outcome carries zero-work cost");
    assert!(matches!(
        unchanged,
        UiMountedFrameOutcome::Unchanged(ref publication) if publication == &first
    ));
    assert_eq!(unchanged_cost.initial_mounted_instances(), 0);
    assert_eq!(unchanged_cost.changed_mounted_instances(), 0);
    assert_eq!(unchanged_cost.index_entries_touched(), 0);
    assert_eq!(unchanged_cost.surface_instance_pairs(), 0);
    assert_eq!(unchanged_cost.adapter().presented_surfaces(), 0);
    assert_eq!(
        unchanged_allocations.count_total, 0,
        "unchanged public mounted frame allocated {unchanged_allocations:?}"
    );
    assert_eq!(
        unchanged_allocations.bytes_total, 0,
        "unchanged public mounted frame allocated {unchanged_allocations:?}"
    );
    assert_eq!(recorder.observed_transcripts().len(), 1);

    let node = first_node(&session);
    session
        .mount_instance(node, surface)
        .expect("one-instance mounted delta is admitted");
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
    let mut changed = None;
    let changed_allocations = allocation_counter::measure(|| {
        changed = Some(
            session
                .execute_mounted_frame(request, UiPresentationDeadline::at_tick(20), 2, |_| {})
                .unwrap_or_else(|_| panic!("changed public mounted frame executes")),
        );
    });
    let changed = changed.expect("allocation observer captures changed outcome");
    let changed_cost = changed
        .cost_report()
        .expect("published changed outcome carries mount and adapter cost");
    let changed_publication = published(changed);
    assert_eq!(changed_publication.predecessor(), Some(first.frame()));
    assert_eq!(changed_cost.changed_mounted_instances(), 1);
    assert_eq!(changed_cost.changed_binding_generations(), 0);
    assert_eq!(changed_cost.adapter().presented_surfaces(), 1);
    assert!(changed_cost.replaced_batch_rows() > 0);
    assert!(changed_cost.replaced_batch_bytes() > 0);
    assert!(
        changed_allocations.count_total <= MAX_CHANGED_PUBLIC_ALLOCATIONS,
        "one-instance public mounted change allocated {} times; independent ceiling is {}",
        changed_allocations.count_total,
        MAX_CHANGED_PUBLIC_ALLOCATIONS
    );
    assert!(
        changed_allocations.bytes_total <= MAX_CHANGED_PUBLIC_BYTES,
        "one-instance public mounted change allocated {} bytes; independent ceiling is {}",
        changed_allocations.bytes_total,
        MAX_CHANGED_PUBLIC_BYTES
    );
    assert_eq!(recorder.observed_transcripts().len(), 2);

    let shutdown = session.shutdown();
    assert!(matches!(
        shutdown.host_session_release(),
        Some(worth_ui_runtime::facade::host::UiHostSessionReleaseOutcome::Released(receipt))
            if receipt.released_surface_count() == 1
    ));
}

fn mount_one(
    session: &mut worth_ui::facade::app::WorthUiActiveApplicationSession,
) -> worth_ui_runtime::facade::mounted::UiSemanticSurfaceIdentity {
    let surface = session
        .create_semantic_surface()
        .expect("semantic surface mints");
    session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .expect("headless surface registers");
    let node = first_node(session);
    session
        .mount_instance(node, surface)
        .expect("one graph node mounts");
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(session);
    surface
}

fn published(
    outcome: UiMountedFrameOutcome,
) -> worth_ui::facade::app::UiMountedFramePublicationReceipt {
    match outcome {
        UiMountedFrameOutcome::Published(publication) => publication,
        _ => panic!("headless public mounted frame must publish"),
    }
}
