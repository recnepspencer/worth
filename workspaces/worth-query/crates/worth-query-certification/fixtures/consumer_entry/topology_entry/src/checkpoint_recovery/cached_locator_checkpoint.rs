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
    let readings = support::custody_calibration::calibrate(|_, _, readings| {
        run_root(WorthQueryOutputDemandResourceProfile::standard(), readings);
    });
    // Four measured Ready units preserve the checkpoint exercise's four-row custody intent.
    let budget = 4
        * (readings
            .at("initial_root")
            .class_bytes("shared_ready_source_continuation")
            - readings
                .at("registered_root")
                .class_bytes("shared_ready_source_continuation"));
    let profile = WorthQueryOutputDemandResourceProfile::standard()
        .with_registry_required_retained_bytes(NonZeroUsize::new(budget).unwrap());
    run_root(
        profile,
        &mut support::custody_calibration::Readings::default(),
    );
}

#[test]
fn a_reclaimed_ready_keeps_its_native_prior_locator_across_checkpoint() {
    let _guard = checkpoint_recovery_test_guard();
    let readings = support::custody_calibration::calibrate(|_, _, readings| {
        run_pair(
            WorthQueryOutputDemandResourceProfile::standard(),
            readings,
            true,
        );
    });
    // The four-row allowance exercises closed-pair reclamation before checkpoint capture.
    let budget = 4
        * (readings
            .at("initial_root")
            .class_bytes("shared_ready_source_continuation")
            - readings
                .at("registered_root")
                .class_bytes("shared_ready_source_continuation"));
    let profile = WorthQueryOutputDemandResourceProfile::standard()
        .with_registry_required_retained_bytes(NonZeroUsize::new(budget).unwrap());
    run_pair(
        profile,
        &mut support::custody_calibration::Readings::default(),
        false,
    );
}

mod calibrated_workloads;
use calibrated_workloads::{run_pair, run_root};

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
