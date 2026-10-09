//! A published output survives the ordinary retirement of its cached Ready row.
use super::*;
mod custody_model;
use worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile;
mod custody_observation;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandProgress, WorthQueryOutputSettlementPosture,
};
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

#[test]
fn an_unreclaimed_root_reopens_under_the_same_custody_profile() {
    let _guard = checkpoint_recovery_test_guard();
    support::capacity_region::search(
        "root checkpoint",
        1,
        128 * 1024,
        support::capacity_region::Goal::Hit,
        run_root,
    )
    .require_hit("root checkpoint");
}

#[test]
fn a_reclaimed_ready_keeps_its_native_prior_locator_across_checkpoint() {
    let _guard = checkpoint_recovery_test_guard();
    support::capacity_region::search(
        "cached pair checkpoint",
        1,
        128 * 1024,
        support::capacity_region::Goal::Hit,
        run_pair,
    )
    .require_hit("cached pair checkpoint");
}

mod capacity_workloads;
use capacity_workloads::{run_pair, run_root};

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
