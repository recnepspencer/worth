



#[allow(unused_imports)]
pub(crate) use {};

#[allow(unused_imports)]
pub(crate) use {};



pub(crate) use {worth_runtime_world::facade::*, crate::court::ordinary_context::CompositeSupplyChainCourt};
pub(crate) use {std::sync::mpsc, std::time::Duration};
pub(crate) use {worth_signal::facade::branch::SignalOwnerOperationBoundaryForTest as Boundary};
pub(crate) const WAIT: Duration = std::time::Duration::from_secs(5);
pub(crate) fn signal_prepared(
    court: &CompositeSupplyChainCourt,
    head: &ProductBranchObservation,
    token: &RuntimeWorldCancellationToken,
) -> PreparedCompositePublicationWithSignal {
    court
        .world
        .publication_port()
        .prepare_with_signal(
            head.clone(),
            CompositePublicationIntent::with_signal(None),
            token,
            None,
        )
        .unwrap()
}
pub(crate) fn clean(court: &CompositeSupplyChainCourt, outcome: RuntimeWorldPublicationOutcome) {
    match outcome {
        RuntimeWorldPublicationOutcome::Performed(done) => drop(done.consume()),
        RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
            let handle = effects.recovery_handle();
            drop(effects);
            let cleanup = court
                .world
                .recovery_port()
                .release_effects(&handle, 0)
                .unwrap();
            assert!(cleanup.owner_retirement_work().is_empty());
        }
        RuntimeWorldPublicationOutcome::NoEffect(no) => drop(no),
    }
}

pub(crate) fn serial_request(
    policy: worth_foundational::ExecutionRequestPolicy,
) -> worth_execution::SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&policy),
        worth_execution::CancellationToken::new(),
        None,
    )
}