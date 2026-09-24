use worth_ui::facade::observation_report::WorthUiHostObservationSessionExt;
use worth_ui::facade::observation_report::{
    UiHostKey, UiHostKeyTransition, UiHostKeyboardModifiers, UiHostObservationCapacity,
    UiHostObservationCapacityInput, UiHostObservationFamily, UiHostObservationLoss,
    UiHostObservationPayload, UiHostObservationReportDenial, UiHostObservationReportOutcome,
};
use worth_ui_runtime::certification_support::ScriptedPresentationOutcome;
use worth_ui_runtime::facade::mounted::{
    UiHostSurfaceCancellationOutcome, UiHostSurfacePresentationMode, UiMountedFrameOutcome,
    UiMountedFrameRetentionBudget, UiMountedFrameRetentionBudgetInput, UiMountedInspectionOmission,
    UiMountedInspectionReceipt, UiMountedInspectionRequest, UiMountedInstanceIdentity,
    UiMountedRetentionClass, UiMountedRetentionClassBudget, UiPresentationDeadline,
    UiSurfaceBindingGeneration,
};
use worth_ui_test_support::WorthUiMountedIdentityCertificationExt;
use worth_ui_test_support::WorthUiMountedPublicationCertificationExt;

use crate::host_observation_fixture::{batch, report, source};
use crate::mounted_application_lifecycle::in_flight_presentation_world::prepared;
use crate::mounted_application_lifecycle::known_empty_surface_world::{
    first_node, mounted_application_with_host_and_capacities, profile,
};
use crate::mounted_application_lifecycle::published_mounted_world::{
    publish, PresentedObservationBasis,
};
use crate::mounted_host_protocol::scripted_host::{presented_completion, ScriptedPresentationHost};

#[path = "retention_saturation/report_assertions.rs"]
mod report_assertions;

const LARGE_STRUCTURAL_BUDGET: usize = 128 * 1024 * 1024;

#[test]
fn sustained_frame_host_and_report_pressure_preserves_bounded_interpretable_truth() {
    let mut world = retention_pressure_world();
    let current = drive_frame_churn(&mut world);
    drive_host_lag(&mut world, current);
    let current = publish(&mut world.session, &world.host, world.instance);
    fill_observation_capacity(&mut world, current);
    fill_quarantine_capacity(&mut world, current);
    report_assertions::assert_bounded_retention_truth(&world, current);
}

struct RetentionPressureWorld {
    session: worth_ui::facade::app::WorthUiActiveApplicationSession,
    host: ScriptedPresentationHost,
    binding: UiSurfaceBindingGeneration,
    instance: UiMountedInstanceIdentity,
}

fn retention_pressure_world() -> RetentionPressureWorld {
    let host = ScriptedPresentationHost::default();
    let mut session = mounted_application_with_host_and_capacities(
        "mounted-retention-saturation",
        host.clone(),
        retention_budget(),
        observation_capacity(),
    )
    .launch()
    .expect("the real filesystem-authored application launches");
    let node = first_node(&session);
    let surface = session.create_semantic_surface().unwrap();
    let binding = session
        .register_host_surface(
            surface,
            UiHostSurfacePresentationMode::RecordOnly,
            profile(1),
        )
        .unwrap()
        .binding_generation();
    session.mount_instance(node, surface).unwrap();
    let instance = session.inspect_mounted_identity().mounted_instances()[0].identity();
    RetentionPressureWorld {
        session,
        host,
        binding,
        instance,
    }
}

fn drive_frame_churn(world: &mut RetentionPressureWorld) -> PresentedObservationBasis {
    let mut current = publish(&mut world.session, &world.host, world.instance);
    for _ in 1..32 {
        current = publish(&mut world.session, &world.host, world.instance);
        let report = world.session.mounted_retention_report();
        assert_eq!(
            report
                .class(UiMountedRetentionClass::Current)
                .retained_items(),
            1
        );
        assert!(
            report
                .class(UiMountedRetentionClass::PredecessorInspection)
                .retained_items()
                <= 2
        );
        report_assertions::assert_current_is_inspectable(&world.session, current);
    }
    current
}

fn drive_host_lag(world: &mut RetentionPressureWorld, predecessor: PresentedObservationBasis) {
    let successor = prepared(&mut world.session);
    let successor_frame = successor.canonical_core().frame();
    world.host.push_in_flight(
        vec![presented_completion()],
        UiHostSurfaceCancellationOutcome::EffectsMayHaveBegun,
    );
    let in_flight = match world.session.present_prepared_mounted_frame(
        successor,
        UiPresentationDeadline::at_tick(1_000),
        0,
    ) {
        UiMountedFrameOutcome::InFlight(in_flight) => in_flight,
        _ => panic!("scripted host lag must retain an in-flight frame"),
    };
    let report = world.session.mounted_retention_report();
    assert_eq!(
        report
            .class(UiMountedRetentionClass::InFlight)
            .retained_items(),
        1
    );
    assert_eq!(
        world
            .session
            .current_mounted_publication()
            .expect("the predecessor publication remains current during host lag")
            .frame(),
        predecessor.frame
    );
    assert!(matches!(
        world
            .session
            .inspect_mounted_frame(UiMountedInspectionRequest::current()),
        UiMountedInspectionReceipt::Omitted(UiMountedInspectionOmission::FrameTransitionInFlight)
    ));
    assert!(matches!(
        world.session.complete_mounted_presentation(in_flight, 1),
        UiMountedFrameOutcome::Published(receipt) if receipt.frame() == successor_frame
    ));
    assert_eq!(
        world
            .session
            .mounted_retention_report()
            .class(UiMountedRetentionClass::InFlight)
            .retained_items(),
        0
    );
    report_assertions::assert_current_frame_is_inspectable(&world.session, successor_frame);
}

fn fill_observation_capacity(world: &mut RetentionPressureWorld, basis: PresentedObservationBasis) {
    for sequence in 1..=2 {
        assert!(matches!(
            world
                .session
                .validate_host_observation_batch(observation_batch(
                    world,
                    basis,
                    sequence,
                    keyboard(sequence),
                )),
            UiHostObservationReportOutcome::Validated(_)
        ));
    }
    assert_eq!(
        world
            .session
            .validate_host_observation_batch(observation_batch(world, basis, 3, keyboard(3),)),
        UiHostObservationReportOutcome::Denied(
            UiHostObservationReportDenial::LocalCapacityExceeded(UiHostObservationFamily::Keyboard)
        )
    );
}

fn fill_quarantine_capacity(
    world: &mut RetentionPressureWorld,
    current: PresentedObservationBasis,
) {
    let candidate = prepared(&mut world.session);
    let indeterminate_frame = candidate.canonical_core().frame();
    world
        .host
        .push_presentation(ScriptedPresentationOutcome::PresentationIndeterminate);
    assert!(matches!(
        world.session.present_prepared_mounted_frame(
            candidate,
            UiPresentationDeadline::at_tick(2_000),
            1,
        ),
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    let basis = PresentedObservationBasis {
        host_surface: current.host_surface,
        frame: indeterminate_frame,
        epoch: current.epoch,
        instance: current.instance,
        receipt: current.receipt,
    };
    assert!(matches!(
        world
            .session
            .validate_host_observation_batch(observation_batch(
                world,
                basis,
                3,
                UiHostObservationPayload::Tick { tick: 3 },
            )),
        UiHostObservationReportOutcome::Quarantined(_)
    ));
    assert_eq!(
        world
            .session
            .validate_host_observation_batch(observation_batch(
                world,
                basis,
                4,
                UiHostObservationPayload::Tick { tick: 4 },
            )),
        UiHostObservationReportOutcome::Denied(
            UiHostObservationReportDenial::QuarantineCountCapacityExceeded
        )
    );
}

fn observation_batch(
    world: &RetentionPressureWorld,
    basis: PresentedObservationBasis,
    sequence: u64,
    payload: UiHostObservationPayload,
) -> worth_ui::facade::observation_report::UiHostObservationBatch {
    batch(
        source(&world.session, world.binding, &basis),
        (sequence, sequence),
        UiHostObservationLoss::Complete,
        vec![report(sequence, payload, &basis)],
    )
}

fn keyboard(sequence: u64) -> UiHostObservationPayload {
    let key = if sequence.is_multiple_of(2) {
        UiHostKey::A
    } else {
        UiHostKey::B
    };
    UiHostObservationPayload::Keyboard {
        logical_key: key,
        physical_key: Some(key),
        modifiers: UiHostKeyboardModifiers::default(),
        transition: if sequence.is_multiple_of(2) {
            UiHostKeyTransition::Pressed { repeat: false }
        } else {
            UiHostKeyTransition::Released
        },
    }
}

fn retention_budget() -> UiMountedFrameRetentionBudget {
    UiMountedFrameRetentionBudget::new(UiMountedFrameRetentionBudgetInput {
        current: UiMountedRetentionClassBudget::new(1, LARGE_STRUCTURAL_BUDGET),
        in_flight: UiMountedRetentionClassBudget::new(1, LARGE_STRUCTURAL_BUDGET),
        observation_basis: UiMountedRetentionClassBudget::new(2, LARGE_STRUCTURAL_BUDGET),
        predecessor_inspection: UiMountedRetentionClassBudget::new(2, LARGE_STRUCTURAL_BUDGET),
        diagnostic: UiMountedRetentionClassBudget::new(0, 0),
        visual_snapshot: UiMountedRetentionClassBudget::new(0, 0),
        visual_overlay: UiMountedRetentionClassBudget::new(0, 0),
        expired_identity_limit: 8,
    })
}

fn observation_capacity() -> UiHostObservationCapacity {
    UiHostObservationCapacity::new(UiHostObservationCapacityInput {
        local_reports: 2,
        local_bytes: 16 * 1024,
        global_reports: 2,
        global_bytes: 32 * 1024,
        quarantined_batches: 1,
        quarantined_bytes: 16 * 1024,
    })
}
