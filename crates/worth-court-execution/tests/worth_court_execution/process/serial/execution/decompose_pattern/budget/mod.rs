pub(crate) use {
    crate::decompose_pattern::ordinary_context::back,
    crate::decompose_pattern::ordinary_context::base_subdomains,
    crate::decompose_pattern::ordinary_context::decomposer,
    crate::decompose_pattern::ordinary_context::interface,
    crate::decompose_pattern::ordinary_context::interior,
    crate::decompose_pattern::ordinary_context::subdomain_map,
    crate::decompose_pattern::ordinary_context::EDITIONS,
};

pub(crate) use {worth_execution::ExecutionAuthorityConfig, worth_execution::LeaseRequest};

#[test]
fn one_scope_enforces_total_work_and_preserves_snapshot_after_stop() {
    if crate::test_process::reexecute_isolated_test() {
        return;
    }

    let authority = worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: std::num::NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: Some(200_000),
    })
    .unwrap();
    let request = |work_ceiling| LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::new(1).unwrap(),
                Some(200_000),
                Some(work_ceiling),
            ),
        ),
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    };
    let entries = base_subdomains();
    let mut decompose = decomposer();
    let too_small = authority.request_lease(request(19)).unwrap();
    let stopped = match decompose.run(
        Some(&too_small),
        &subdomain_map(&entries),
        EDITIONS,
        interior,
        interface,
        back,
    ) {
        Ok(_) => panic!("work ceiling should stop back-substitution"),
        Err(stopped) => stopped,
    };
    assert!(matches!(
        stopped.cause,
        worth_execution::DecomposeFailure::Back { .. }
    ));
    assert_eq!(stopped.total_report.charged_work(), 19);
    assert_eq!(
        stopped.total_report.fallback(),
        Some(worth_foundational::ExecutionFallbackCause::PolicySerial),
        "a stopped decomposition keeps its stages' fallback"
    );

    let sufficient = authority.request_lease(request(20)).unwrap();
    let complete = decompose
        .run(
            Some(&sufficient),
            &subdomain_map(&entries),
            EDITIONS,
            interior,
            interface,
            back,
        )
        .unwrap();
    assert_eq!(complete.total_report.charged_work(), 20);
    assert_eq!(
        complete.total_report.fallback(),
        Some(worth_foundational::ExecutionFallbackCause::PolicySerial)
    );
    let accounting = complete
        .reduction_metrics
        .accounted_metrics()
        .expect("a policy-serial run carries reduction accounting");
    assert_eq!(accounting.structural_visits, 11);
    assert_eq!(accounting.combine_calls, 4);
    assert_eq!(accounting.charged_work, 15);
    assert_eq!(complete.reuse.back_substitutions_reused, 0);
}
