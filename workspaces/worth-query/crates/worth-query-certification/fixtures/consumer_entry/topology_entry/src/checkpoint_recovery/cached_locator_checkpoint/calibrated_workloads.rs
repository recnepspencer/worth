//! Ample and pressured runs use the same checkpoint lifecycle.
use super::*;
pub(super) fn run_root(
    profile: WorthQueryOutputDemandResourceProfile,
    readings: &mut support::custody_calibration::Readings,
) {
    let application = support::install_program_with_seed::<CheckpointProgram>(
        None,
        profile,
        32,
        128 * 1_024 * 1_024,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        support::seed_cycle,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut original = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<CheckpointProgram, CheckpointRoot>(&application)
        .unwrap();
    readings.record(
        "registered_root",
        support::custody_calibration::Inventory::new(
            application.required_custody_breakdown_for_test(),
            Vec::new(),
            application.output_lineage_retained_bytes_for_test(),
        ),
    );
    assert_eq!(
        settle(&mut original, &request).producer_contacts_in_this_demand(),
        1
    );
    readings.record(
        "initial_root",
        support::custody_calibration::Inventory::new(
            application.required_custody_breakdown_for_test(),
            Vec::new(),
            application.output_lineage_retained_bytes_for_test(),
        ),
    );
    drop(original);
    drop(principal);
    drop(scope);
    let checkpoint = application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap();
    drop(application);
    let reopened = support::install_program::<CheckpointProgram>(Some(checkpoint), profile);
    let (scope, principal) = authenticate(&reopened);
    let request = reopened.request(&principal, &scope);
    let mut restored = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<CheckpointProgram, CheckpointRoot>(&reopened)
        .unwrap();
    readings.record(
        "restored_start",
        support::custody_calibration::Inventory::new(
            reopened.required_custody_breakdown_for_test(),
            Vec::new(),
            reopened.output_lineage_retained_bytes_for_test(),
        ),
    );
    assert_eq!(
        settle(&mut restored, &request).producer_contacts_in_this_demand(),
        0,
        "the constrained profile itself must preserve ordinary checkpoint reuse"
    );
    readings.record(
        "restored_root",
        support::custody_calibration::Inventory::new(
            reopened.required_custody_breakdown_for_test(),
            Vec::new(),
            reopened.output_lineage_retained_bytes_for_test(),
        ),
    );
}

pub(super) fn run_pair(
    profile: WorthQueryOutputDemandResourceProfile,
    readings: &mut support::custody_calibration::Readings,
    calibration: bool,
) {
    let application = support::install_program_with_seed::<CheckpointProgram>(
        None,
        profile,
        32,
        128 * 1_024 * 1_024,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        support::seed_cycle,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut first = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<CheckpointProgram, CheckpointRoot>(&application)
        .unwrap();
    readings.record(
        "registered_root",
        support::custody_calibration::Inventory::new(
            application.required_custody_breakdown_for_test(),
            Vec::new(),
            application.output_lineage_retained_bytes_for_test(),
        ),
    );
    let original = settle(&mut first, &request);
    assert_eq!(
        original.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    assert_eq!(original.producer_contacts_in_this_demand(), 1);
    readings.record(
        "initial_root",
        support::custody_calibration::Inventory::new(
            application.required_custody_breakdown_for_test(),
            Vec::new(),
            application.output_lineage_retained_bytes_for_test(),
        ),
    );
    drop((first, original));
    let mut final_initial = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<CheckpointProgram, FinalConnection>(&application)
        .unwrap();
    let final_result = settle_final(&mut final_initial, &request);
    let final_entity = final_result
        .outputs_of::<FinalPlanarOutputs>()
        .unwrap()
        .entity::<FinalAnchorOutput<CheckpointSchema>>()
        .unwrap()
        .entity_id();
    drop((final_initial, final_result));

    // Two disjoint real root publications each need Ready custody. The
    // installed profile has finite custody, so they displace closed cached
    // output cells without deleting either native body.
    for key in ["anchor-isolated", "anchor-island"] {
        let mut other = request
            .demand(PlanarOutputDemand::new(key))
            .start_in_program::<CheckpointProgram, CheckpointRoot>(&application)
            .unwrap();
        if key == "anchor-island" {
            readings.record(
                "fourth_start",
                support::custody_calibration::Inventory::new(
                    application.required_custody_breakdown_for_test(),
                    Vec::new(),
                    application.output_lineage_retained_bytes_for_test(),
                ),
            );
        }
        let other_result = settle(&mut other, &request);
        if key == "anchor-isolated" {
            readings.record(
                "third_open",
                support::custody_calibration::Inventory::new(
                    application.required_custody_breakdown_for_test(),
                    Vec::new(),
                    application.output_lineage_retained_bytes_for_test(),
                ),
            );
        }
        assert_eq!(
            other_result.posture(),
            WorthQueryOutputSettlementPosture::Performed
        );
        drop((other, other_result));
    }
    if calibration {
        return;
    }
    let prior = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(prior.rows()[0].value, length(2));
    let final_prior = request
        .query(PlanarOutputRead {
            body_key: "final:anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(final_prior.rows()[0].value, length(3));
    drop(final_prior);
    drop(prior);
    drop(principal);
    drop(scope);
    let checkpoint = application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap();
    drop(application);

    let reopened = support::install_program::<CheckpointProgram>(Some(checkpoint), profile);
    let (scope, principal) = authenticate(&reopened);
    let request = reopened.request(&principal, &scope);
    let mut demand = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<CheckpointProgram, FinalConnection>(&reopened)
        .unwrap();
    let restored = settle_final(&mut demand, &request);
    let preserved = restored
        .outputs_of::<FinalPlanarPreserveOutputs>()
        .expect("a prior-only locator must select Preserve rather than duplicate Initial");
    assert_eq!(
        preserved
            .entity::<FinalPreservedAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        final_entity,
        "Preserve must retain the native output entity"
    );
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "final:anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(3)
    );
    drop(restored);
    drop(demand);
    drop(principal);
    drop(scope);
    let preserved_checkpoint = reopened
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap();
    drop(reopened);

    let reopened_again =
        support::install_program::<CheckpointProgram>(Some(preserved_checkpoint), profile);
    let (scope, principal) = authenticate(&reopened_again);
    let request = reopened_again.request(&principal, &scope);
    let mut repeated = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<CheckpointProgram, FinalConnection>(&reopened_again)
        .unwrap();
    let repeated_result = settle_final(&mut repeated, &request);
    let repeated_preserve = repeated_result
        .outputs_of::<FinalPlanarPreserveOutputs>()
        .expect("a checkpoint of performed Preserve must select Preserve again");
    assert_eq!(
        repeated_preserve
            .entity::<FinalPreservedAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        final_entity,
        "repeated Preserve keeps the original native entity"
    );
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "final:anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(3)
    );
}
