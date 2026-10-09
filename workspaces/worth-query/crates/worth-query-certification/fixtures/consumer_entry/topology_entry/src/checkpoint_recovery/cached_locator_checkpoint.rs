//! A published output survives the ordinary retirement of its cached Ready row.
use super::*;
mod custody_model;
mod custody_observation;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryOutputSettlementPosture,
};
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

#[test]
fn an_unreclaimed_root_reopens_under_the_same_custody_profile() {
    let _guard = checkpoint_recovery_test_guard();
    let profile = custody_model::single_root_profile();
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
    assert_eq!(
        settle(&mut original, &request).producer_contacts_in_this_demand(),
        1
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
    assert_eq!(
        settle(&mut restored, &request).producer_contacts_in_this_demand(),
        0,
        "the constrained profile itself must preserve ordinary checkpoint reuse"
    );
}

#[test]
fn a_reclaimed_ready_keeps_its_native_prior_locator_across_checkpoint() {
    let _guard = checkpoint_recovery_test_guard();
    let profile = custody_model::reclamation_profile();
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
    let original = settle(&mut first, &request);
    assert_eq!(
        original.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    assert_eq!(original.producer_contacts_in_this_demand(), 1);
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
        let other_result = settle(&mut other, &request);
        assert_eq!(
            other_result.posture(),
            WorthQueryOutputSettlementPosture::Performed
        );
        drop((other, other_result));
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

fn settle_final<'application>(
    demand: &mut worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandHandle<
        'application,
        CheckpointSchema,
        PlanarFinalOutputDemand,
    >,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        '_,
        '_,
        CheckpointSchema,
    >,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandSettlement<
    PlanarQuery,
> {
    (0..64)
        .find_map(|_| match demand.advance(request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
        })
        .expect("the ordinary final-output demand settles")
}

fn settle<'application>(
    demand: &mut worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandHandle<
        'application, CheckpointSchema, PlanarOutputDemand,
    >,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        '_,
        '_,
        CheckpointSchema,
    >,
) -> worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandSettlement<
    PlanarQuery,
> {
    (0..64)
        .find_map(|_| match demand.advance(request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
        })
        .expect("the ordinary native demand settles")
}
