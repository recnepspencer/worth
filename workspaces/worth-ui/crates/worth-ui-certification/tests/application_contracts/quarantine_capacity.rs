use worth_ui::facade::observation_report::WorthUiHostObservationSessionExt;
use worth_ui::facade::observation_report::{
    UiHostObservationCapacity, UiHostObservationCapacityInput, UiHostObservationLoss,
    UiHostObservationPayload, UiHostObservationReportDenial, UiHostObservationReportOutcome,
};
use worth_ui_runtime::facade::mounted::{
    UiHostSurfacePresentationMode, UiMountedFrameOutcome, UiMountedFrameRetentionBudget,
    UiMountedRetentionClass, UiPresentationDeadline, UiSurfaceBindingGeneration,
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
use crate::mounted_host_protocol::scripted_host::ScriptedPresentationHost;

#[test]
fn quarantine_count_budget_denies_without_partial_retention() {
    let mut world = indeterminate_world(
        "observation-quarantine-count-budget",
        observation_capacity(1, 64 * 1024),
    );
    assert!(matches!(
        world.validate(1),
        UiHostObservationReportOutcome::Quarantined(_)
    ));
    let retained_bytes = world.session.quarantined_host_observation_byte_count();
    assert!(retained_bytes > 0);
    assert_eq!(
        world.validate(2),
        UiHostObservationReportOutcome::Denied(
            UiHostObservationReportDenial::QuarantineCountCapacityExceeded
        )
    );
    assert_eq!(world.session.quarantined_host_observation_batch_count(), 1);
    assert_eq!(
        world.session.quarantined_host_observation_byte_count(),
        retained_bytes
    );

    let report = world.session.mounted_retention_report();
    let quarantine = report.class(UiMountedRetentionClass::Quarantine);
    assert_eq!(quarantine.retained_items(), 1);
    assert_eq!(quarantine.retained_structural_bytes(), retained_bytes);
    let budget = quarantine
        .queue_budget()
        .expect("quarantine is governed by a queue budget");
    assert_eq!(budget.item_limit(), 1);
    assert_eq!(budget.structural_byte_limit(), 64 * 1024);
}

#[test]
fn quarantine_byte_budget_denies_the_first_real_entry_atomically() {
    let mut world = indeterminate_world(
        "observation-quarantine-byte-budget",
        observation_capacity(8, 0),
    );
    assert_eq!(
        world.validate(1),
        UiHostObservationReportOutcome::Denied(
            UiHostObservationReportDenial::QuarantineByteCapacityExceeded
        )
    );
    assert_eq!(world.session.quarantined_host_observation_batch_count(), 0);
    assert_eq!(world.session.quarantined_host_observation_byte_count(), 0);
    let quarantine = world
        .session
        .mounted_retention_report()
        .class(UiMountedRetentionClass::Quarantine)
        .clone();
    assert_eq!(quarantine.retained_items(), 0);
    assert_eq!(quarantine.retained_structural_bytes(), 0);
}

struct IndeterminateObservationWorld {
    session: worth_ui::facade::app::WorthUiActiveApplicationSession,
    binding: UiSurfaceBindingGeneration,
    basis: PresentedObservationBasis,
}

impl IndeterminateObservationWorld {
    fn validate(&mut self, sequence: u64) -> UiHostObservationReportOutcome {
        let raw = batch(
            source(&self.session, self.binding, &self.basis),
            (sequence, sequence),
            UiHostObservationLoss::Complete,
            vec![report(
                sequence,
                UiHostObservationPayload::Tick { tick: sequence },
                &self.basis,
            )],
        );
        self.session.validate_host_observation_batch(raw)
    }
}

fn indeterminate_world(
    label: &str,
    observation_capacity: UiHostObservationCapacity,
) -> IndeterminateObservationWorld {
    let host = ScriptedPresentationHost::default();
    let mut session = mounted_application_with_host_and_capacities(
        label,
        host.clone(),
        UiMountedFrameRetentionBudget::default(),
        observation_capacity,
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
    let current = publish(&mut session, &host, instance);
    let indeterminate = prepared(&mut session);
    let frame = indeterminate.canonical_core().frame();
    host.push_presentation(
        worth_ui_runtime::certification_support::ScriptedPresentationOutcome::PresentationIndeterminate,
    );
    assert!(matches!(
        session.present_prepared_mounted_frame(
            indeterminate,
            UiPresentationDeadline::at_tick(1_000),
            0,
        ),
        UiMountedFrameOutcome::PresentationIndeterminate(_)
    ));
    let host_surface =
        session.inspect_mounted_identity().surface_bindings()[0].host_surface_identity();
    IndeterminateObservationWorld {
        session,
        binding,
        basis: PresentedObservationBasis {
            host_surface,
            frame,
            epoch: current.epoch,
            instance,
            receipt: current.receipt,
        },
    }
}

fn observation_capacity(
    quarantined_batches: usize,
    quarantined_bytes: usize,
) -> UiHostObservationCapacity {
    UiHostObservationCapacity::new(UiHostObservationCapacityInput {
        local_reports: 64,
        local_bytes: 16 * 1024,
        global_reports: 512,
        global_bytes: 128 * 1024,
        quarantined_batches,
        quarantined_bytes,
    })
}
