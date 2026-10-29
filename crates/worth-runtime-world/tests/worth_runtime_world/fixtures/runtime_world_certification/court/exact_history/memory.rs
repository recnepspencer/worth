// Actual World source/parent backing follows serial and inherited request custody.
pub(crate) use super::*;
pub(crate) use worth_execution::{
    CancellationSource, ExecutionRequest, LeaseDenial, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};

pub(crate) fn assert_retained_memory(
    court: &CompositeSupplyChainCourt,
    target: &ProductBranchObservation,
) {
    for inherited in [false, true] {
        let budget = SerialMemoryBudget::new(Some(1_000_000));
        let parent =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        let worker = SerialRequest::from_policy(
            &worth_foundational::ExecutionRequestPolicy::for_workers(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                std::num::NonZeroUsize::MIN,
            ),
            CancellationSource::new().token(),
            None,
        );
        let retained = if inherited {
            assert!(!ExecutionRequest::serial(&worker).has_accounting());
            ExecutionRequest::serial(&parent)
                .run_native(|_| {
                    retain_page(court, target, &budget, ExecutionRequest::serial(&worker))
                })
                .unwrap()
                .0
        } else {
            retain_page(court, target, &budget, ExecutionRequest::serial(&parent))
        };
        assert!(
            budget.charged_memory_bytes() > 0,
            "successor pins remain funded after request scope exit"
        );
        assert_cancelled_successor(court, &retained.1, &budget);
        drop(retained);
        assert_eq!(
            budget.charged_memory_bytes(),
            0,
            "last World page and cursor release source/position backing"
        );
    }
    let budget = SerialMemoryBudget::new(Some(0));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let refused = court
        .world
        .observation_port()
        .open_exact_product_history(ExecutionRequest::serial(&host), target);
    assert!(
        matches!(
            refused,
            Err(RuntimeWorldServiceDenial::Denied(
                ProductExactHistoryDenial::Request(WorkCeilingDenial::Admission(
                    LeaseDenial::MemoryExhausted(_)
                ))
            ))
        ),
        "actual zero World request refusal: {refused:?}"
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
}

pub(crate) fn retain_page(
    court: &CompositeSupplyChainCourt,
    target: &ProductBranchObservation,
    budget: &SerialMemoryBudget,
    execution: ExecutionRequest<'_, '_>,
) -> (ProductExactHistoryPage, ProductExactHistoryCursor) {
    let port = court.world.observation_port();
    let cursor = port.open_exact_product_history(execution, target).unwrap();
    let source_charge = budget.charged_memory_bytes();
    assert!(source_charge > 0);
    let prepared = port
        .prepare_exact_product_history_page(execution, &cursor)
        .unwrap();
    assert!(
        budget.charged_memory_bytes() > source_charge,
        "actual next parent allocation is funded"
    );
    drop(prepared);
    assert_eq!(budget.charged_memory_bytes(), source_charge);
    let prepared = port
        .prepare_exact_product_history_page(execution, &cursor)
        .unwrap();
    let retained = cursor.accept_page(prepared).unwrap();
    drop(cursor);
    retained
}

pub(crate) fn assert_cancelled_successor(
    court: &CompositeSupplyChainCourt,
    cursor: &ProductExactHistoryCursor,
    budget: &SerialMemoryBudget,
) {
    let port = court.world.observation_port();
    let source_charge = budget.charged_memory_bytes();
    let cancellation = CancellationSource::new();
    let host = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let prepared = port
        .prepare_exact_product_history_page(ExecutionRequest::serial(&host), cursor)
        .unwrap();
    assert!(
        !prepared.is_complete(),
        "cancelled preparation still reaches a real raw parent"
    );
    assert!(
        budget.charged_memory_bytes() > source_charge,
        "actual next parent is funded"
    );
    cancellation.cancel();
    assert!(matches!(
        cursor.accept_page(prepared),
        Err(ProductExactHistoryDenial::Request(
            WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled)
        ))
    ));
    assert_eq!(
        budget.charged_memory_bytes(),
        source_charge,
        "denied successor releases only staged backing"
    );
}
