use worth_ui::facade::app::{
    WorthUiMountedApplicationReplacementOutcome, WorthUiMountedReplacementPreparationOutcome,
};
use worth_ui::facade::measurement_exchange::WorthUiHostMeasurementSessionExt;
use worth_ui::facade::measurement_exchange::{
    UiFontMeasurementKey, UiHostMeasurementDeadline, UiHostMeasurementDenial,
    UiHostMeasurementIntent, UiHostMeasurementObservation, UiHostMeasurementObservationValue,
    UiHostMeasurementOutcome, UiHostMeasurementRequest, UiHostMeasurementRequestIntent,
    UiMeasurementEvidenceFamily, UiTextIntrinsicSizeObservation, UiTextIntrinsicSizeRequest,
};
use worth_ui::facade::observation_report::WorthUiHostObservationSessionExt;
use worth_ui::facade::observation_report::{UiHostObservationLoss, UiHostObservationReportOutcome};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameRequest, UiPresentationDeadline,
};
use worth_ui_test_support::{
    WorthUiActiveSessionCertificationExt, WorthUiMountedIdentityCertificationExt,
};

use super::host_measurement_fixture::{
    begin_portal, begin_text, begin_viewport, mounted_measurement_session, portal_observation,
    text_observation, viewport_observation,
};
use super::host_observation_fixture::{batch, report, source};
use super::mounted_application_lifecycle::known_empty_surface_world::profile;
use super::mounted_application_lifecycle::published_mounted_world::{
    presented_epoch, PresentedObservationBasis,
};
use super::mounted_publication::{replacement_workspace, stage_replacement};

#[test]
fn owner_issued_requests_complete_reorder_cancel_and_expire() {
    let (_, mut session, bindings) = mounted_measurement_session("host-measurement-lifecycle", 1);
    let first = begin_viewport(&mut session, Some(bindings[0]), 100, 0);
    let second = begin_viewport(&mut session, Some(bindings[0]), 100, 0);

    assert!(matches!(
        session.complete_host_measurement(viewport_observation(&second, 800.0, 600.0), 1),
        UiHostMeasurementOutcome::Completed(_)
    ));
    assert_eq!(
        session.cancel_host_measurement(first.identity()),
        UiHostMeasurementOutcome::Cancelled(first.identity())
    );
    assert_eq!(
        session.complete_host_measurement(viewport_observation(&first, 640.0, 480.0), 2),
        UiHostMeasurementOutcome::DuplicateSuppressed(first.identity())
    );

    let expiring = begin_viewport(&mut session, Some(bindings[0]), 5, 3);
    assert_eq!(
        session.expire_host_measurements(5).as_ref(),
        &[UiHostMeasurementOutcome::Expired(expiring.identity())]
    );
}

#[test]
fn real_adapter_environment_drift_terminalizes_exact_dependencies() {
    let (host, mut session, bindings) =
        mounted_measurement_session("host-measurement-environment", 1);
    let viewport = begin_viewport(&mut session, Some(bindings[0]), 100, 0);
    host.advance_viewport_environment();
    assert_eq!(
        session.complete_host_measurement(viewport_observation(&viewport, 800.0, 600.0), 1),
        UiHostMeasurementOutcome::Denied(UiHostMeasurementDenial::StaleBasis)
    );

    let viewport_twin = begin_viewport(&mut session, Some(bindings[0]), 100, 2);
    assert!(matches!(
        session.complete_host_measurement(viewport_observation(&viewport_twin, 800.0, 600.0), 3,),
        UiHostMeasurementOutcome::Completed(_)
    ));

    let text = begin_text(&mut session, Some(bindings[0]), "Inbox", 100, 4);
    host.advance_font_environment();
    assert_eq!(
        session.complete_host_measurement(text_observation(&text, 80.0, 20.0), 5),
        UiHostMeasurementOutcome::Denied(UiHostMeasurementDenial::StaleBasis)
    );
}

#[test]
fn real_wui_lifecycle_closes_phase_one_through_eight_authority_seams() {
    let (host, mut session, bindings) =
        mounted_measurement_session("host-measurement-allocation", 1);
    assert_eq!(host.native_registration_count(), 1);
    let predecessor = begin_portal(&mut session, bindings[0], 100, 0);
    crate::mounted_geometry_fixture::install_current_occurrence_geometry(&mut session);
    let workspace = replacement_workspace("host-measurement-allocation");
    let (pending, catalog, boundary) = stage_replacement(&workspace, &mut session);
    let replacement = match session
        .prepare_mounted_replacement(
            pending,
            catalog,
            boundary,
            None,
            UiMountedFrameRequest::all_bound_surfaces(),
        )
        .expect("real filesystem replacement should prepare")
    {
        WorthUiMountedReplacementPreparationOutcome::Prepared(replacement) => replacement,
        WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(_) => {
            panic!("changed allocation catalog must prepare a successor")
        }
    };
    assert_eq!(replacement.frame().manifest().surfaces().len(), 1);
    assert!(!replacement.frame().surfaces()[0]
        .projection()
        .nodes()
        .is_empty());
    host.push_presented();
    let (application, mounted) = match replacement.present(UiPresentationDeadline::at_tick(20), 1) {
        WorthUiMountedApplicationReplacementOutcome::Published {
            application,
            mounted,
        } => (application, mounted),
        _ => panic!("presented replacement must publish atomically"),
    };
    assert!(!application
        .allocation_catalog_successor()
        .transitions()
        .is_empty());
    assert_eq!(session.current_mounted_publication(), Some(&mounted));

    assert_eq!(
        session.complete_host_measurement(portal_observation(&predecessor), 1),
        UiHostMeasurementOutcome::Denied(UiHostMeasurementDenial::StaleBasis)
    );
    let successor = begin_portal(&mut session, bindings[0], 100, 2);
    assert!(matches!(
        session.complete_host_measurement(portal_observation(&successor), 3),
        UiHostMeasurementOutcome::Completed(_)
    ));
    let identity = session.inspect_mounted_identity();
    let instance = identity.mounted_instances()[0].identity();
    let frame = identity
        .current_frame()
        .expect("replacement frame is current");
    let basis = PresentedObservationBasis {
        host_surface: identity.surface_bindings()[0].host_surface_identity(),
        frame,
        epoch: presented_epoch(&session, frame, bindings[0]),
        instance,
        receipt: identity
            .frame_receipts()
            .iter()
            .find(|receipt| receipt.mounted_instance_identity() == instance)
            .expect("current mounted instance has a frame receipt")
            .node_receipt_identity(),
    };
    drop(identity);
    let observation = batch(
        source(&session, bindings[0], &basis),
        (1, 1),
        UiHostObservationLoss::Complete,
        vec![report(
            1,
            crate::host_observation_fixture::window_focus(&basis, true),
            &basis,
        )],
    );
    assert!(matches!(
        session.validate_host_observation_batch(observation),
        UiHostObservationReportOutcome::Validated(_)
    ));
    let rebound = session
        .rebind_host_surface(
            bindings[0],
            UiHostSurfacePresentationMode::RecordOnly,
            profile(9),
        )
        .expect("exact native deregistration and registration rebind");
    assert_ne!(rebound.binding_generation(), bindings[0]);
    assert_eq!(host.native_registration_count(), 1);
    let _ = session.shutdown();
    workspace.close();
}

#[test]
fn malformed_same_identity_completion_does_not_consume_pending_request() {
    let (_, mut session, bindings) = mounted_measurement_session("host-measurement-malformed", 1);
    let requested = begin_text(&mut session, Some(bindings[0]), "Inbox", 100, 0);
    let counterfeit = UiHostMeasurementRequest::text_intrinsic_size(
        requested.identity(),
        UiMeasurementEvidenceFamily::TextIntrinsicSize,
        UiTextIntrinsicSizeRequest::single_line("Different", UiFontMeasurementKey::new("body")),
        session.host_measurement_capability().capability_report(),
    )
    .unwrap();
    let malformed = UiHostMeasurementObservation::from_request(
        &counterfeit,
        UiHostMeasurementObservationValue::TextIntrinsicSize(UiTextIntrinsicSizeObservation {
            width: 1.0,
            height: 1.0,
        }),
    )
    .unwrap();

    assert_eq!(
        session.complete_host_measurement(malformed, 1),
        UiHostMeasurementOutcome::Denied(UiHostMeasurementDenial::StaleBasis)
    );
    assert_eq!(session.pending_host_measurement_count(), 1);
    assert!(matches!(
        session.complete_host_measurement(text_observation(&requested, 80.0, 20.0), 2),
        UiHostMeasurementOutcome::Completed(_)
    ));
}

#[test]
fn real_surface_rebind_terminalizes_bound_measurement() {
    let (_, mut session, bindings) = mounted_measurement_session("host-measurement-rebind", 1);
    let requested = begin_viewport(&mut session, Some(bindings[0]), 100, 0);
    let successor = session
        .rebind_host_surface(
            bindings[0],
            UiHostSurfacePresentationMode::RecordOnly,
            profile(2),
        )
        .unwrap()
        .binding_generation();
    assert_ne!(successor, bindings[0]);
    assert_eq!(
        session.complete_host_measurement(viewport_observation(&requested, 1_200.0, 800.0), 1),
        UiHostMeasurementOutcome::Denied(UiHostMeasurementDenial::StaleBasis)
    );
    assert_eq!(session.pending_host_measurement_count(), 0);
}

#[test]
fn measurement_budgets_backpressure_without_counterfeit_identity() {
    let (_, mut session, bindings) = mounted_measurement_session("host-measurement-bounds", 1);
    let mut admitted = Vec::new();
    for _ in 0..64 {
        admitted.push(begin_viewport(&mut session, Some(bindings[0]), 100, 0));
    }
    assert_eq!(session.pending_host_measurement_count(), 64);
    assert_eq!(
        session.begin_host_measurement(
            UiHostMeasurementIntent::new(
                Some(bindings[0]),
                UiHostMeasurementRequestIntent::viewport_extent(
                    worth_ui::facade::measurement_exchange::UiViewportExtentRequest,
                ),
                UiHostMeasurementDeadline::at_tick(100),
            ),
            0,
        ),
        UiHostMeasurementOutcome::Denied(UiHostMeasurementDenial::CapacityExceeded)
    );
    for requested in admitted {
        let _ = session.cancel_host_measurement(requested.identity());
    }

    let oversized = UiHostMeasurementIntent::new(
        Some(bindings[0]),
        UiHostMeasurementRequestIntent::text_intrinsic_size(
            UiTextIntrinsicSizeRequest::single_line(
                "x".repeat(70 * 1024),
                UiFontMeasurementKey::new("body"),
            ),
        ),
        UiHostMeasurementDeadline::at_tick(100),
    );
    assert_eq!(
        session.begin_host_measurement(oversized, 0),
        UiHostMeasurementOutcome::Denied(UiHostMeasurementDenial::ByteCapacityExceeded)
    );
    assert_eq!(session.pending_host_measurement_count(), 0);
    assert_eq!(session.pending_host_measurement_bytes(), 0);
}
