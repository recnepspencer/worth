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
    assert_classes(&application, 1, 0, 1, 0);
    drop((first, root));
    assert_classes(&application, 1, 0, 0, 0);
    let mut final_initial = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<CheckpointProgram, FinalConnection>(&application)
        .unwrap();
    let final_result = settle_final(&mut final_initial, &request);
    assert_classes(&application, 1, 1, 0, 1);
    drop((final_initial, final_result));
    assert_classes(&application, 1, 1, 0, 0);
}

fn assert_classes(
    application: &support::Application,
    roots: usize,
    finals: usize,
    root_members: usize,
    final_members: usize,
) {
    let actual: Vec<_> = application
        .required_custody_breakdown_for_test()
        .into_iter()
        .map(|(class, _, bytes)| (class, bytes))
        .collect();
    assert_eq!(
        actual,
        custody_model::expected_classes(roots, finals, root_members, final_members),
        "every settled class has its layout-derived owner"
    );
}
