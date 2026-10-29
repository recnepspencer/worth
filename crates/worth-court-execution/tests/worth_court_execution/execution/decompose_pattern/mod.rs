#[allow(unused_imports)] use crate::decompose_pattern::ordinary_context::*;
pub(crate) use {worth_execution::BackInput, worth_execution::DecomposeFailure, worth_execution::DecomposeInputDenial, worth_execution::DecomposeKernelEditions, worth_execution::ExecutionDecompose, worth_execution::ExecutionMap, worth_execution::InterfaceSolution, worth_execution::InteriorResult, worth_execution::MapKernelFailure, worth_execution::MapPartition};
pub(crate) use {worth_foundational::PartitionIdentity};

#[test]
fn sparse_interface_solve_reuses_unchanged_contribution_and_local_slice() {
    let entries = base_subdomains();
    let mut decompose = decomposer();
    let initial = decompose
        .run(
            None,
            &subdomain_map(&entries),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(initial.total_report.charged_work(), 20);
    let serial = decomposer()
        .run(
            None,
            &subdomain_map(&entries),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    for (actual, expected) in initial.values.iter().zip(&serial.values) {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
    let direct = direct_solve(&[entries[0].1, entries[1].1]);
    assert!((initial.values[0] - direct[0]).abs() < 1e-12);
    assert!((initial.values[1] - direct[2]).abs() < 1e-12);
    assert!((direct[1] - 1.0).abs() < 1e-12);

    let repeated = decompose
        .run(
            None,
            &subdomain_map(&[]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(repeated.reuse.unchanged_contributions, 2);
    assert!(repeated.reuse.interface_solve_reused);
    assert_eq!(repeated.reuse.back_substitutions_reused, 2);
    assert!(repeated.interface_report.is_none());
    assert!(repeated.back_report.is_none());

    let mut changed = entries[0].1;
    changed.rhs += 2.0;
    changed.base_rhs += 1.0;
    let updated = decompose
        .run(
            None,
            &subdomain_map(&[(1, changed)]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(updated.reuse.unchanged_contributions, 2);
    assert!(updated.reuse.interface_solve_reused);
    assert_eq!(updated.reuse.back_substitutions_reused, 1);
    assert!(updated.back_report.is_some());
    let serial = decomposer()
        .run(
            None,
            &subdomain_map(&[(1, changed), entries[1]]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    for (actual, expected) in updated.values.iter().zip(&serial.values) {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }

    changed.rhs += 1.0;
    let affected = decompose
        .run(
            None,
            &subdomain_map(&[(1, changed)]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(affected.reuse.unchanged_contributions, 1);
    assert!(!affected.reuse.interface_solve_reused);
    assert_eq!(affected.reuse.back_substitutions_reused, 0);
    let serial = decomposer()
        .run(
            None,
            &subdomain_map(&[(1, changed), entries[1]]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    for (actual, expected) in affected.values.iter().zip(&serial.values) {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
}

#[test]
fn stopped_stage_does_not_publish_a_partial_snapshot() {
    let entries = base_subdomains();
    let mut decompose = decomposer();
    let initial = decompose
        .run(
            None,
            &subdomain_map(&entries),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    let failure = decompose.run(
        None,
        &subdomain_map(&[(1, entries[0].1)]),
        EDITIONS,
        |_value,
         _context|
         -> Result<InteriorResult<Interior, InterfaceEquation>, MapKernelFailure<()>> {
            panic!("test kernel panic")
        },
        interface,
        back,
    );
    assert!(
        matches!(failure, Err(error) if matches!(error.cause, worth_execution::DecomposeFailure::Interior { .. }))
    );
    let after = decompose
        .run(
            None,
            &subdomain_map(&[]),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    for (actual, expected) in after.values.iter().zip(&initial.values) {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
    assert_eq!(after.reuse.back_substitutions_reused, 2);
}
#[allow(unused_imports)]
pub(crate) use {};




#[path = "editions/mod.rs"]
pub(crate) mod editions;
#[path = "tree_reuse/mod.rs"]
pub(crate) mod tree_reuse;
