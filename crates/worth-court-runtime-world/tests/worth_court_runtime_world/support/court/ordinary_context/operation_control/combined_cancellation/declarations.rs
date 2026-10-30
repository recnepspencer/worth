pub(crate) use {
    crate::court::ordinary_context::operation_control::serial_request,
    crate::court::ordinary_context::CompositeSupplyChainCourt, worth_runtime_world::facade::*,
};

pub(crate) fn combined_candidate(
    court: &CompositeSupplyChainCourt,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    root: &ProductBranchObservation,
    token: &RuntimeWorldCancellationToken,
) -> PreparedCompositePublicationWithSignal {
    let candidate =
        court
            .records
            .candidate(execution, root.basis().relational_basis(), "grain", "5");
    court
        .world
        .publication_port()
        .prepare_with_signal(
            root.clone(),
            CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
                .with_prepared_relational_candidate(candidate),
            token,
            None,
        )
        .unwrap()
}
