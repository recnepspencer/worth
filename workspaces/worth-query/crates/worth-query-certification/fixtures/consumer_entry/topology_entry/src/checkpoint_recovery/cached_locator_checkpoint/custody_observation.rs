//! Observer-only inventory of the initial pair; capacities are not selected from measurements.
use super::*;
use worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile;

#[test]
fn cached_checkpoint_initial_pair_reports_owner_custody_classes() {
    let _guard = checkpoint_recovery_test_guard();
    let application = support::install_program::<CheckpointProgram>(
        None,
        WorthQueryOutputDemandResourceProfile::standard(),
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut first = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<CheckpointProgram, CheckpointRoot>(&application)
        .unwrap();
    let root = settle(&mut first, &request);
    eprintln!(
        "CACHED_ROOT_OPEN {:?}",
        application.required_custody_breakdown_for_test()
    );
    drop((first, root));
    eprintln!(
        "CACHED_ROOT_CLOSED {:?}",
        application.required_custody_breakdown_for_test()
    );
    let mut final_initial = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<CheckpointProgram, FinalConnection>(&application)
        .unwrap();
    let final_result = settle_final(&mut final_initial, &request);
    eprintln!(
        "CACHED_PAIR_OPEN {:?}",
        application.required_custody_breakdown_for_test()
    );
    drop((final_initial, final_result));
    eprintln!(
        "CACHED_PAIR_CLOSED {:?}",
        application.required_custody_breakdown_for_test()
    );
    custody_model::report_models();
}
