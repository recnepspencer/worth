use crate::certification_support::{ScriptedPresentationHost, ScriptedSurfaceCompletion};
use crate::facade::mounted::{UiHostSurfaceCancellationOutcome, UiMountedFrameOutcome};
use crate::facade::mounted::{
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedOccurrenceGeometry, UiMountedSurfaceGeometryBatch,
};
use crate::runtime::tests::active_application_session_test_support::source_backed_component_app_with_host;
use crate::runtime::tests::active_application_session_test_support::source_backed_declared_surface_component_app_with_host;
use crate::runtime::{WorthUiReloadDebounce, WorthUiSourceProvider, WorthUiWatcherEvent};

use super::{WorthUiNativeManagedSourceRebindOutcome, WorthUiNativeSourceRebindDenial};

#[test]
fn managed_source_rebind_remains_owned_until_host_progress_or_shutdown() {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_component_app_with_host(host.clone())
        .launch_native_surface()
        .expect("source-backed native fixture should launch");
    super::native_application_identity_trace_test_support::install_bound_surface_geometry(
        &mut shell,
    );
    assert!(matches!(
        shell.present_frame(100, 1),
        Ok(UiMountedFrameOutcome::Published(_))
    ));

    host.push_in_flight(
        vec![ScriptedSurfaceCompletion::Pending],
        UiHostSurfaceCancellationOutcome::CancelledBeforeEffects,
    );
    let first_request = source_request(&shell, 2);
    let outcome = shell
        .begin_source_rebind_with_layout(first_request, candidate_test_layout)
        .expect("source successor should reach managed presentation");
    assert!(matches!(
        outcome,
        WorthUiNativeManagedSourceRebindOutcome::Pending
    ));
    assert_eq!(host.native_in_flight_count(), 1);

    let second_request = source_request(&shell, 3);
    let denial = match shell.begin_source_rebind(second_request) {
        Err(denial) => denial,
        Ok(_) => panic!("pending source work must retain the sole managed slot"),
    };
    assert!(matches!(
        denial,
        WorthUiNativeSourceRebindDenial::ManagedRebindAlreadyInFlight
    ));
    let shutdown = shell.shutdown();
    assert!(shutdown.host_session_released());
    assert_eq!(host.native_in_flight_count(), 0);
}

#[test]
fn public_source_rebind_exposes_only_current_mounted_rows_after_publication() {
    const SUCCESSOR: &str = "component:workspace.component.active_session_candidate";
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_component_app_with_host(host.clone())
        .launch_native_surface()
        .expect("source-backed native fixture should launch");
    super::native_application_identity_trace_test_support::install_bound_surface_geometry(
        &mut shell,
    );
    assert!(matches!(
        shell.present_frame(100, 1),
        Ok(UiMountedFrameOutcome::Published(_))
    ));
    let predecessor = shell.native_component_layout_inputs()[0]
        .authored_semantic_identity()
        .to_owned();

    host.push_native_display_settled_without_effects();
    let request = source_request(&shell, 2);
    let mut supplied_geometry = None;
    match shell.begin_source_rebind_with_layout(request, |input| {
        assert_eq!(input.components().len(), 1);
        assert_eq!(
            input.components()[0].authored_semantic_identity(),
            SUCCESSOR
        );
        let instance = input.components()[0].instance();
        let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: 24.0,
            y: 32.0,
            width: 640.0,
            height: 360.0,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .expect("application-owned candidate geometry should be canonical");
        supplied_geometry = Some((instance, bounds, input.revision()));
        Some(UiMountedSurfaceGeometryBatch::new(
            input.basis(),
            input.revision(),
            input.viewport(),
            [UiMountedOccurrenceGeometry::surface(instance, bounds)],
        ))
    }) {
        Ok(WorthUiNativeManagedSourceRebindOutcome::Published(_)) => {}
        Ok(WorthUiNativeManagedSourceRebindOutcome::Pending) => {
            let posture = match shell.pending_managed_rebind.as_ref() {
                Some(super::native_managed_rebind::WorthUiNativePendingManagedRebind::Retry {
                    requires_reconstruction,
                    ..
                }) => format!("retry(reconstruction={requires_reconstruction})"),
                Some(super::native_managed_rebind::WorthUiNativePendingManagedRebind::Completion(_)) => "completion".to_owned(),
                Some(super::native_managed_rebind::WorthUiNativePendingManagedRebind::Indeterminate { .. }) => "indeterminate".to_owned(),
                Some(super::native_managed_rebind::WorthUiNativePendingManagedRebind::RecoveryReconstruction { .. }) => "recovery-reconstruction".to_owned(),
                Some(super::native_managed_rebind::WorthUiNativePendingManagedRebind::RecoveryReconstructionDeferred(_)) => "recovery-deferred".to_owned(),
                Some(super::native_managed_rebind::WorthUiNativePendingManagedRebind::PredecessorReconstruction { .. }) => "predecessor-reconstruction".to_owned(),
                Some(_) => "non-rebind".to_owned(),
                None => "missing".to_owned(),
            };
            panic!("synchronous trace host unexpectedly retained the source rebind: {posture}")
        }
        Ok(WorthUiNativeManagedSourceRebindOutcome::Stopped(stop)) => {
            panic!("source replacement stopped: {stop:?}")
        }
        Err(denial) => panic!("source replacement denied: {denial:?}"),
    }
    assert_eq!(shell.mounted_component_instance(&predecessor), None);
    let successor = shell
        .mounted_component_instance(SUCCESSOR)
        .expect("accepted source successor must be mounted immediately");
    let layout = shell.native_component_layout_inputs();
    assert_eq!(layout.len(), 1);
    assert_eq!(layout[0].authored_semantic_identity(), SUCCESSOR);
    assert_eq!(layout[0].instance(), successor);
    let (laid_out, bounds, revision) = supplied_geometry.expect("candidate layout must run");
    assert_eq!(laid_out, successor);
    assert_eq!(
        [bounds.x(), bounds.y(), bounds.width(), bounds.height()],
        [24.0, 32.0, 640.0, 360.0]
    );
    assert_eq!(
        shell.next_native_layout_revision().unwrap().get(),
        revision.get().saturating_add(1)
    );
    assert!(!shell.native_program_layout_required());

    assert!(shell.shutdown().host_session_released());
}

#[test]
fn source_rebind_without_candidate_layout_preserves_the_predecessor_with_typed_denial() {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_component_app_with_host(host.clone())
        .launch_native_surface()
        .expect("source-backed native fixture should launch");
    super::native_application_identity_trace_test_support::install_bound_surface_geometry(
        &mut shell,
    );
    assert!(matches!(
        shell.present_frame(100, 1),
        Ok(UiMountedFrameOutcome::Published(_))
    ));
    let predecessor = shell.native_component_layout_inputs()[0].instance();
    host.push_native_display_settled_without_effects();

    let request = source_request(&shell, 2);
    let denial = match shell.begin_source_rebind(request) {
        Err(denial) => denial,
        Ok(_) => panic!("replacement without candidate layout must be denied"),
    };
    assert!(matches!(
        denial,
        WorthUiNativeSourceRebindDenial::Preparation(
            crate::runtime::rebind::UiRebindPreparationDenial::CandidateMountedPreparation(
                denial
            )
        ) if matches!(
            denial.as_ref(),
            crate::mounting::UiMountedFramePreparationDenial::Projection(
                crate::mounting::UiMountedProjectionDenial::OccurrenceGeometry(
                    crate::mounting::UiMountedOccurrenceGeometryDenial::MissingOccurrenceGeometry
                )
            )
        )
    ));
    assert_eq!(
        shell.native_component_layout_inputs()[0].instance(),
        predecessor
    );
    assert!(!shell.native_program_layout_required());
    assert!(shell.shutdown().host_session_released());
}

#[test]
fn candidate_source_replacement_derives_mosaic_clip_from_candidate_geometry() {
    let host = ScriptedPresentationHost::native_display();
    host.push_native_display_presented();
    let mut shell = source_backed_declared_surface_component_app_with_host(host.clone())
        .launch_native_declared_surface("workspace.surface.native")
        .expect("declared Mosaic surface should launch");
    install_declared_surface_layout(
        &mut shell,
        1,
        [0.0, 0.0, 640.0, 480.0],
        [8.0, 12.0, 300.0, 180.0],
    );
    assert!(matches!(
        shell.present_frame(100, 1),
        Ok(UiMountedFrameOutcome::Published(_))
    ));

    host.push_native_display_settled_without_effects();
    let request = declared_surface_source_request(&shell, 2);
    let expected_clip = box_at(
        44.0,
        54.0,
        240.0,
        120.0,
        UiMountedCoordinateSpace::HostSurface,
    );
    let outcome = shell
        .begin_source_rebind_with_layout(request, |input| {
            let component = input.components().first()?;
            let region = input.regions().first()?;
            assert_eq!(region.owner(), component.instance());
            let bounds = box_at(
                32.0,
                36.0,
                500.0,
                320.0,
                UiMountedCoordinateSpace::HostSurface,
            );
            let local_region = box_at(
                12.0,
                18.0,
                240.0,
                120.0,
                UiMountedCoordinateSpace::GraphNodeLocal,
            );
            Some(
                UiMountedSurfaceGeometryBatch::new(
                    input.basis(),
                    input.revision(),
                    input.viewport(),
                    [UiMountedOccurrenceGeometry::surface(
                        component.instance(),
                        bounds,
                    )],
                )
                .with_regions([region.geometry(local_region)]),
            )
        })
        .expect("candidate geometry should validate and publish");
    assert!(matches!(
        outcome,
        WorthUiNativeManagedSourceRebindOutcome::Published(_)
    ));
    let successor = shell
        .mounted_component_instance("component:workspace.component.active_session_candidate")
        .expect("candidate component should be current");
    assert_eq!(
        shell
            .session
            .mounted
            .current_mosaic_clips_for_test(successor)
            .expect("candidate geometry row should exist")
            .as_ref(),
        &[expected_clip]
    );
    assert!(shell.shutdown().host_session_released());
}

fn install_declared_surface_layout(
    shell: &mut super::WorthUiNativeApplicationShell,
    revision: u64,
    occurrence: [f32; 4],
    region: [f32; 4],
) {
    let component = shell.native_component_layout_inputs()[0].clone();
    let declared_region = shell.native_region_layout_inputs()[0].clone();
    let batch = UiMountedSurfaceGeometryBatch::new(
        shell.native_layout_basis().unwrap(),
        crate::mounting::UiMountedLayoutRevision::new(revision).unwrap(),
        box_at(
            0.0,
            0.0,
            1280.0,
            720.0,
            UiMountedCoordinateSpace::HostSurface,
        ),
        [UiMountedOccurrenceGeometry::surface(
            component.instance(),
            box_at(
                occurrence[0],
                occurrence[1],
                occurrence[2],
                occurrence[3],
                UiMountedCoordinateSpace::HostSurface,
            ),
        )],
    )
    .with_regions([declared_region.geometry(box_at(
        region[0],
        region[1],
        region[2],
        region[3],
        UiMountedCoordinateSpace::GraphNodeLocal,
    ))]);
    shell.complete_native_layout(batch).unwrap();
}

fn box_at(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    coordinate_space: UiMountedCoordinateSpace,
) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space,
    })
    .unwrap()
}

fn candidate_test_layout(
    input: super::UiNativeReplacementLayoutInput,
) -> Option<UiMountedSurfaceGeometryBatch> {
    let component = input.components().first()?;
    let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 0.0,
        y: 0.0,
        width: input.viewport().width(),
        height: input.viewport().height(),
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .ok()?;
    Some(UiMountedSurfaceGeometryBatch::new(
        input.basis(),
        input.revision(),
        input.viewport(),
        [UiMountedOccurrenceGeometry::surface(
            component.instance(),
            bounds,
        )],
    ))
}

fn source_request(
    shell: &super::WorthUiNativeApplicationShell,
    tick: u64,
) -> crate::runtime::rebind::UiSourceRebindRequest {
    let provider = WorthUiSourceProvider::in_memory(format!("managed-source-{tick}")).with_file(
        "app/main.wui",
        "component workspace.component.active_session_candidate { region workspace.region.primary { sizing workspace.sizing.mosaic_support; } }",
    );
    let events = [WorthUiWatcherEvent::provider_revision(provider.id())];
    let snapshot = WorthUiReloadDebounce::default()
        .debounce(provider, &events, tick)
        .expect("complete in-memory source should settle");
    crate::runtime::rebind::UiSourceRebindRequest::new(snapshot)
        .with_deadline(shell.rebind_deadline_at(tick.saturating_add(10)))
        .observed_at_tick(tick)
}

fn declared_surface_source_request(
    shell: &super::WorthUiNativeApplicationShell,
    tick: u64,
) -> crate::runtime::rebind::UiSourceRebindRequest {
    let provider = WorthUiSourceProvider::in_memory(format!("declared-mosaic-source-{tick}"))
        .with_file(
            "app/main.wui",
            "surface workspace.surface.native {}\ncomponent workspace.component.active_session_candidate { region workspace.region.primary { sizing workspace.sizing.mosaic_support; } }",
        );
    let events = [WorthUiWatcherEvent::provider_revision(provider.id())];
    let snapshot = WorthUiReloadDebounce::default()
        .debounce(provider, &events, tick)
        .expect("declared candidate source should settle");
    crate::runtime::rebind::UiSourceRebindRequest::new(snapshot)
        .with_deadline(shell.rebind_deadline_at(tick.saturating_add(10)))
        .observed_at_tick(tick)
}
