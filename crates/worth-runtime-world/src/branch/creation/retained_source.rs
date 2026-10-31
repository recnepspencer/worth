//! Exact retained truth for creation; this is not application navigation authority.
use crate::branch::ProductBranchObservation;
use worth_relational::facade::branch::AdmittedRelationalRetainedForkSource;

/// Move-only source admitted by this World's branch service. Both components
/// must be independently forked; Signal still uses its exact current receiver.
/// Attached Native durability is unsupported before any destination effects.
/// The ordinary receiver requires a current observation:
/// ```compile_fail,E0308
/// use worth_runtime_world::facade::*;
/// fn current(port: &RuntimeWorldBranchPort, execution: &worth_execution::ExecutionRequest<'_, '_>, source: RetainedProductBranchCreationSource, intent: ProductBranchCreationIntent, cancellation: &RuntimeWorldCancellationToken) {
///     let _ = port.create_product_branch(execution, source, intent, cancellation);
/// }
/// ```
/// Admit the held observation and consume that exact retained source:
/// ```
/// use worth_runtime_world::facade::*;
/// fn retained(port: &RuntimeWorldBranchPort, execution: &worth_execution::ExecutionRequest<'_, '_>, source: &ProductBranchObservation, intent: ProductBranchCreationIntent, cancellation: &RuntimeWorldCancellationToken)
///     -> Result<RuntimeWorldBranchCreationOutcome, RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>> {
///     let source = port.admit_retained_branch_source(source)?;
///     port.create_product_branch_from_retained_source(execution, source, intent, cancellation)
/// }
/// ```
#[derive(Debug)]
pub struct RetainedProductBranchCreationSource {
    pub(crate) observation: ProductBranchObservation,
    pub(crate) native: AdmittedRelationalRetainedForkSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BranchCreationSourceKind {
    Current,
    Retained,
}
