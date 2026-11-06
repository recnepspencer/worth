//! Actual historical payload custody for a new occurrence on a current branch.
pub(in crate::mvcc) mod payload;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::identity::data::EntityId;
use crate::mvcc::BranchBoundRelationalTransaction;

/// One owner-admitted historical entity selection on a pristine current
/// transaction. This retains both actual roots; a description cannot mint it.
/// Application navigation permission and domain eligibility are separate.
///
/// Transfer an original pristine current transaction and retained source to
/// the selection owner, then use the ordinary candidate publication/discard
/// services. Preparation retains the historical root and publishes nothing.
///
/// ```
/// use worth_execution::ExecutionRequest;
/// use worth_relational::facade::{
///     branch::AdmittedRelationalBranchBasis, identity::EntityId,
///     history::CanonicalRetainedEntitySelection,
///     mvcc::{BranchBoundRelationalTransaction, PreparedRelationalCommitCandidate,
///         RelationalPreparationPort, RelationalRetainedEntitySelectionPreparationError as Error},
/// };
/// fn prepare_selection(
///     port: &RelationalPreparationPort, execution: &ExecutionRequest<'_, '_>,
///     current: BranchBoundRelationalTransaction,
///     historical: &AdmittedRelationalBranchBasis, entity: EntityId,
/// ) -> Result<PreparedRelationalCommitCandidate, Error> {
///     let selection = port.admit_retained_entity_selection(current, historical, entity)
///         .map_err(Error::Admission)?;
///     port.prepare_retained_entity_selection(execution, selection)
/// }
/// fn selected_entity(provenance: &CanonicalRetainedEntitySelection) -> EntityId {
///     provenance.entity()
/// }
/// ```
///
/// Selection custody cannot be supplied as an ordinary transaction.
///
/// ```compile_fail,E0308
/// use worth_execution::ExecutionRequest;
/// use worth_relational::facade::mvcc::{
///     RelationalPreparationPort, RelationalRetainedEntitySelection,
/// };
/// fn wrong_receiver(port: &RelationalPreparationPort,
///     execution: &ExecutionRequest<'_, '_>, selection: RelationalRetainedEntitySelection) {
///     port.prepare_branch_transaction(execution, selection).unwrap();
/// }
/// ```
#[derive(Debug)]
pub struct RelationalRetainedEntitySelection {
    pub(crate) transaction: BranchBoundRelationalTransaction,
    pub(crate) historical: AdmittedRelationalBranchBasis,
    pub(crate) entity: EntityId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRetainedEntitySelectionDenial {
    OwnerUnavailable,
    ForeignRuntime,
    TargetNotCurrent,
    TargetNotPristine,
    HistoricalSourceRetired,
    HistoricalSourceUnavailable,
    SchemaChanged,
    IdentityChanged,
    UnsupportedIdentityMeaning,
    DurabilityUnsupported,
}

/// Preparation refused before publication; the original publication service
/// remains the only receiver that can perform the resulting candidate.
#[derive(Debug)]
pub enum RelationalRetainedEntitySelectionPreparationError {
    Admission(RelationalRetainedEntitySelectionDenial),
    Preparation(crate::transactions::data::TransactionCommitError),
}
