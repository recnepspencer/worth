//! One ample execution records the settled chain and its refreshed inventory.
use super::*;
use support::custody_calibration::{calibrate, Inventory, Readings};

pub(super) fn chain() -> Readings {
    calibrate(|required, index, readings| {
        let (application, invalidation) = limited_application(required, index, 8);
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let mut a = request
            .demand(PlanarOutputDemand::new("anchor-a"))
            .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
            .unwrap();
        readings.record(
            "before_root",
            Inventory::new(
                application.required_custody_breakdown_for_test(),
                invalidation.retained_custody_breakdown_for_test(),
                application.output_lineage_retained_bytes_for_test(),
            ),
        );
        settle!(a, request);
        readings.record(
            "settled_root",
            Inventory::new(
                application.required_custody_breakdown_for_test(),
                invalidation.retained_custody_breakdown_for_test(),
                application.output_lineage_retained_bytes_for_test(),
            ),
        );
        let mut b = request
            .demand(ChainDemand("anchor-b".to_owned()))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                &application,
            )
            .unwrap();
        settle!(b, request);
        let mut c = request
            .demand(ChainDemand("anchor-c".to_owned()))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                &application,
            )
            .unwrap();
        settle!(c, request);
        readings.record(
            "settled_chain",
            Inventory::new(
                application.required_custody_breakdown_for_test(),
                invalidation.retained_custody_breakdown_for_test(),
                application.output_lineage_retained_bytes_for_test(),
            ),
        );
        drop((a, b));
        readings.record(
            "lone_caller",
            Inventory::new(
                application.required_custody_breakdown_for_test(),
                invalidation.retained_custody_breakdown_for_test(),
                application.output_lineage_retained_bytes_for_test(),
            ),
        );
        change_root_input!(request, application, 2, 0x9176_4000_u64);
        settle!(c, request);
        readings.record(
            "refreshed_chain",
            Inventory::new(
                application.required_custody_breakdown_for_test(),
                invalidation.retained_custody_breakdown_for_test(),
                application.output_lineage_retained_bytes_for_test(),
            ),
        );
        drop(c);
        readings.record(
            "after_reclamation",
            Inventory::new(
                application.required_custody_breakdown_for_test(),
                invalidation.retained_custody_breakdown_for_test(),
                application.output_lineage_retained_bytes_for_test(),
            ),
        );
    })
}

pub(super) fn ready_unit(readings: &Readings) -> usize {
    readings
        .at("settled_root")
        .class_bytes("shared_ready_source_continuation")
        - readings
            .at("before_root")
            .class_bytes("shared_ready_source_continuation")
}
