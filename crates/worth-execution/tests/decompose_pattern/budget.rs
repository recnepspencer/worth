use super::*;
use std::num::NonZeroUsize;

use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionFallbackCause, ExecutionPosture,
    ExecutionRequestPolicy,
};

#[test]
fn one_scope_enforces_total_work_and_preserves_snapshot_after_stop() {
    let authority = ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(2).unwrap(),
        charged_memory_bytes: Some(200_000),
    })
    .unwrap();
    let request = |work_ceiling| LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                NonZeroUsize::new(1).unwrap(),
                Some(200_000),
                Some(work_ceiling),
            ),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
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
    assert!(matches!(stopped.cause, DecomposeFailure::Back { .. }));
    assert_eq!(stopped.total_report.charged_work(), 19);
    assert_eq!(
        stopped.total_report.fallback(),
        Some(ExecutionFallbackCause::PolicySerial),
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
        Some(ExecutionFallbackCause::PolicySerial)
    );
    let accounting = complete
        .reduction_metrics
        .accounted_metrics()
        .expect("an accounted lease records reduction accounting");
    assert_eq!(accounting.structural_visits, 11);
    assert_eq!(accounting.combine_calls, 4);
    assert_eq!(accounting.charged_work, 15);
    assert_eq!(complete.reuse.back_substitutions_reused, 0);
}
