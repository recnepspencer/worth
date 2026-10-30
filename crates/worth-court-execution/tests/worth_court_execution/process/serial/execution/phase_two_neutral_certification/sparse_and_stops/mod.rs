#[allow(unused_imports)]
use crate::phase_two_neutral_certification::isolated_context::sparse_and_stops::*;
use {
    crate::phase_two_neutral_certification::isolated_context::execution_matrix,
    crate::phase_two_neutral_certification::isolated_context::lease,
    crate::phase_two_neutral_certification::isolated_context::EDITIONS,
    crate::phase_two_neutral_certification::isolated_context::EQUIVALENCE_ID,
    crate::phase_two_neutral_certification::isolated_context::TEST_LOCK,
    worth_execution::compare_canonical_values,
};

#[test]
fn sparse_decomposition_matches_independent_undecomposed_solver_across_widths() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let values = domains();
    let expected = undecomposed_solve(&values);
    assert_eq!(expected, [2.0, 1.0, 2.0]);
    let mut serial_bits = None;
    let mut serial_work = None;
    for (posture, width) in execution_matrix() {
        let lease = lease(
            posture,
            width,
            20,
            worth_execution::CancellationToken::new(),
            None,
            worth_foundational::DeterminismContract::ContractEquivalent(EQUIVALENCE_ID),
        );
        assert_eq!(
            lease.policy().determinism(),
            worth_foundational::DeterminismContract::ContractEquivalent(EQUIVALENCE_ID)
        );
        let mut decompose = decomposition();
        let result = decompose
            .certify(
                &lease,
                0x9e37_79b9 ^ width as u64,
                &domain_map(&values),
                EDITIONS,
                interior,
                interface,
                back,
            )
            .unwrap();
        let bits = result
            .values
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>();
        if let Some(serial) = &serial_bits {
            assert_eq!(&bits, serial);
        } else {
            serial_bits = Some(bits);
        }
        assert!(compare_canonical_values(&lease, &expected[0], &result.values[0]).unwrap());
        assert!(compare_canonical_values(&lease, &expected[2], &result.values[1]).unwrap());
        let work = result.total_report.charged_work();
        assert_eq!(
            work,
            result.interior_report.charged_work()
                + result
                    .reduction_metrics
                    .accounted_metrics()
                    .expect("an accounted decomposition carries reduction accounting")
                    .charged_work
                + result.interface_report.unwrap().charged_work()
                + result.back_report.unwrap().charged_work()
        );
        if let Some(serial) = serial_work {
            assert_eq!(work, serial);
        } else {
            serial_work = Some(work);
        }
        assert!(result.total_report.charged_span() < result.total_report.charged_work());

        let changed = Subdomain {
            rhs: values[0].rhs + 1.0,
            ..values[0]
        };
        let updated = decompose
            .certify(
                &lease,
                0xdead_beef ^ width as u64,
                &changed_domain_map(changed),
                EDITIONS,
                interior,
                interface,
                back,
            )
            .unwrap();
        let direct = undecomposed_solve(&[changed, values[1]]);
        assert!(compare_canonical_values(&lease, &direct[0], &updated.values[0]).unwrap());
        assert!(compare_canonical_values(&lease, &direct[2], &updated.values[1]).unwrap());
    }
    let bitwise = lease(
        worth_foundational::ExecutionPosture::Serial,
        1,
        1,
        worth_execution::CancellationToken::new(),
        None,
        worth_foundational::DeterminismContract::CanonicalBitwise,
    );
    let adjacent = f64::from_bits(expected[0].to_bits() + 1);
    assert!(!compare_canonical_values(&bitwise, &expected[0], &adjacent).unwrap());
    let equivalent = lease(
        worth_foundational::ExecutionPosture::Serial,
        1,
        1,
        worth_execution::CancellationToken::new(),
        None,
        worth_foundational::DeterminismContract::ContractEquivalent(EQUIVALENCE_ID),
    );
    assert!(compare_canonical_values(&equivalent, &expected[0], &adjacent).unwrap());
    assert!(!compare_canonical_values(&equivalent, &expected[0], &(expected[0] + 1e-3)).unwrap());

    let ceiling = serial_work.unwrap() - 1;
    let limited = lease(
        worth_foundational::ExecutionPosture::Serial,
        1,
        ceiling,
        worth_execution::CancellationToken::new(),
        None,
        worth_foundational::DeterminismContract::CanonicalBitwise,
    );
    let exhausted = decomposition().run(
        Some(&limited),
        &domain_map(&values),
        EDITIONS,
        interior,
        interface,
        back,
    );
    assert!(matches!(exhausted, Err(error)
        if matches!(error.cause, worth_execution::DecomposeFailure::Back { .. })
            && error.total_report.charged_work() == ceiling));
}

#[path = "stage_stops/mod.rs"]
pub(crate) mod stage_stops;
