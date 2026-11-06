use super::*;

pub(super) fn fork(
    court: &CompositeSupplyChainCourt,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &ProductBranchObservation,
) -> ProductBranchObservation {
    let plans = crate::court::ordinary_context::branches::plans(true, true, "candidate");
    match court
        .world
        .branch_port()
        .create_product_branch(
            execution,
            source.clone(),
            ProductBranchCreationIntent::from_source("candidate", plans).unwrap(),
            &RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap()
    {
        RuntimeWorldBranchCreationOutcome::Performed(branch) => branch,
        other => panic!("genuine candidate fork: {other:?}"),
    }
}

pub(super) fn intent<S>(
    mut intent: CompositePublicationIntent<S>,
    current: &ProductBranchObservation,
) -> CompositePublicationIntent<S> {
    intent.require_current_read(current.clone()).unwrap();
    intent
}

pub(super) fn prepare(
    court: &CompositeSupplyChainCourt,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    target: &ProductBranchObservation,
    current: &ProductBranchObservation,
) -> PreparedCompositePublicationWithoutSignal {
    let candidate =
        court
            .records
            .candidate(execution, target.basis().relational_basis(), "grain", "5");
    court
        .world
        .publication_port()
        .prepare_without_signal(
            target.clone(),
            intent(
                CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                    .with_prepared_relational_candidate(candidate),
                current,
            ),
            &RuntimeWorldCancellationSource::new().token(),
            None,
        )
        .unwrap()
}

pub(super) fn move_signal(
    court: &CompositeSupplyChainCourt,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &ProductBranchObservation,
) -> ProductBranchObservation {
    let token = RuntimeWorldCancellationSource::new().token();
    let prepared = court
        .world
        .publication_port()
        .prepare_with_signal(
            source.clone(),
            CompositePublicationIntent::with_signal(None),
            &token,
            None,
        )
        .unwrap();
    let outcome = court.world.publication_port().execute_with_signal(
        *execution,
        prepared,
        &mut court.context(execution, source),
        &token,
        |_| Ok(()),
    );
    match outcome {
        RuntimeWorldPublicationOutcome::Performed(done) => drop(done.consume()),
        other => panic!("genuine Signal successor: {other:?}"),
    }
    court.observe(source)
}

pub(super) fn grain(
    court: &CompositeSupplyChainCourt,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    head: &ProductBranchObservation,
) -> String {
    court
        .records
        .read(execution, head.basis().relational_basis())
        .records["grain"]
        .clone()
}

pub(super) fn direct_basis(
    court: &CompositeSupplyChainCourt,
    head: &ProductBranchObservation,
) -> worth_relational::facade::branch::AdmittedRelationalBranchBasis {
    court
        .records
        .runtime
        .observe_branch(head.basis().relational_basis().identity())
        .unwrap()
        .1
}
