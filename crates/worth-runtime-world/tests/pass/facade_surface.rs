use worth_runtime_world::facade::{
    AdmittedRelationalBranchBasis, AdmittedRuntimeWorldCorrespondenceBasis,
    AdmittedSignalBranchBasis, CompositeAttemptProgress, PreparedCompositePublicationWithSignal,
    PreparedCompositePublicationWithoutSignal, ProductBranchObservation,
    RelationalOwnerServicePorts, RelationalTransactionIntent, RuntimeWorldCancellationSource,
    RuntimeWorldCancellationToken, RuntimeWorldCorrespondencePort, RuntimeWorldOwner,
    RuntimeWorldPublicationOutcome, SignalError, SignalOwnerCancellationToken,
    SignalOwnerServicePorts, SignalTransaction,
};

fn takes_public_return_types(
    progress: &CompositeAttemptProgress,
    expected: &ProductBranchObservation,
    outcome: &RuntimeWorldPublicationOutcome,
) {
    let _ = progress.relational_posture();
    let _ = expected.selected_commit();
    let _ = format!("{outcome:?}");
}

fn names_public_component_signatures(
    _relational: &RelationalOwnerServicePorts,
    _signal: &SignalOwnerServicePorts<(), (), (), u32, ()>,
    _bridge: &RuntimeWorldCorrespondencePort,
    _relational_basis: &AdmittedRelationalBranchBasis,
    _signal_basis: &AdmittedSignalBranchBasis,
    _correspondence_basis: &AdmittedRuntimeWorldCorrespondenceBasis,
    _intent: Option<RelationalTransactionIntent>,
    _token: Option<SignalOwnerCancellationToken>,
    _world_inputs: Option<RuntimeWorldOwner<(), (), (), u32, ()>>,
) {
    let _error: Option<SignalError> = None;
    let _transaction: Option<SignalTransaction<'static, (), (), (), u32, ()>> = None;
}

/// The two prepared stages are separate public types. A caller can name both
/// without any conversion between them existing.
fn names_both_prepared_stages(
    _without_signal: Option<PreparedCompositePublicationWithoutSignal>,
    _with_signal: Option<PreparedCompositePublicationWithSignal>,
) {
}

/// The Signal execution seam is an unboxed `FnOnce` carrying exactly the owner
/// port's bound, so a public caller names it without a trait object.
fn names_the_signal_execution_seam<F>(_mutation: F)
where
    F: FnOnce(&mut SignalTransaction<'_, (), (), (), u32, ()>) -> Result<(), SignalError>,
{
}

fn main() {
    let cancellation = RuntimeWorldCancellationSource::new();
    let _token: RuntimeWorldCancellationToken = cancellation.token();
    names_the_signal_execution_seam(|_transaction| Ok(()));
    let _ = (
        takes_public_return_types,
        names_public_component_signatures,
        names_both_prepared_stages,
    );
}

// Type-check the complete public construction/execution boundary under exactly
// the predecessor's bounds, with no Clone, Default or Debug on E or Ctx.
fn exact_generic_contract<D, I, E, Ctx, T, F>(
    relational: RelationalOwnerServicePorts,
    signal: SignalOwnerServicePorts<D, I, E, Ctx, T>,
    signal_definition_publication: worth_signal::facade::branch::SignalConditionalDefinitionPublicationPort<
        D, I, E, Ctx, T,
    >,
    bridge: RuntimeWorldCorrespondencePort,
    budgets: worth_runtime_world::facade::RuntimeWorldBudgets,
    clock: worth_runtime_world::facade::RuntimeWorldClock,
    prepared: PreparedCompositePublicationWithSignal,
    context: &mut Ctx,
    cancellation: &RuntimeWorldCancellationToken,
    apply: F,
) -> RuntimeWorldPublicationOutcome
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
    F: FnOnce(&mut SignalTransaction<'_, D, I, E, Ctx, T>) -> Result<(), SignalError>,
{
    let owner = RuntimeWorldOwner::builder()
        .with_bridge_correspondence(bridge)
        .with_relational_services(relational)
        .with_signal_services(signal)
        .with_signal_definition_publication(signal_definition_publication)
        .with_budgets(budgets)
        .with_clock(clock)
        .build()
        .unwrap();
    owner
        .publication_port()
        .execute_with_signal(prepared, context, cancellation, apply)
}

fn canonical_owner_artifacts(
    performed: &worth_runtime_world::facade::PerformedCompositePublication,
) {
    let results = performed.component_results();
    let _: Option<&worth_runtime_world::facade::CommitResult> = results.relational_commit_result();
    let _: Option<&worth_runtime_world::facade::RelationalCommitReceipt> =
        results.relational_settlement();
    let _: Option<&worth_runtime_world::facade::SignalBranchAdvanceOutcome> =
        results.signal().advanced_outcome();
    let _: Option<&worth_runtime_world::facade::SignalBranchForkOutcome> =
        results.signal().fork_outcome();
}
