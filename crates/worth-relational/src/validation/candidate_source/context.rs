//! Per-validation metadata is bound to each real Native proposal round.
use super::{NativeManagedSourceInterpretation, NativeSourceInterpretationIdentity};
use std::sync::Arc;
mod share;
pub use share::{ManagedIndexSourceContextReceipt, ManagedIndexSourceContextShare};

/// A caller's metadata factory, borrowed only during validation. Native invokes
/// it once for each actual interpretation, including discarded retry rounds.
/// The caller retains strong children through the entire invocation and retires
/// rejected children outside its runtime guard. Native retains weak receipts only.
/// The returned metadata conveys no READ, Source or publication authority.
/// An outer registration fence must be owned separately by the caller: retained
/// or abandoned Native drafts cannot determine that fence's lifetime.
pub trait ManagedIndexSourceContextFactory: Send + Sync {
    fn bind(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        interpretation: NativeSourceInterpretationIdentity,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
    ) -> Result<ManagedIndexSourceContextReceipt, super::ManagedIndexSourcePreparationDenial>;
}

pub(super) struct BoundSourceContext(ManagedIndexSourceContextReceipt);
impl std::fmt::Debug for BoundSourceContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoundSourceContext").finish_non_exhaustive()
    }
}

/// Borrowed from the actual interpretation. Identity comparison is descriptive;
/// no constructor, root upgrade, READ or publication door is exposed.
pub struct ManagedIndexSourcePreparationMetadata<'a> {
    identity: NativeSourceInterpretationIdentity,
    payload: &'a ManagedIndexSourceContextReceipt,
}
impl ManagedIndexSourcePreparationMetadata<'_> {
    pub fn interpretation(&self) -> &NativeSourceInterpretationIdentity {
        &self.identity
    }
    pub fn payload(&self) -> &ManagedIndexSourceContextReceipt {
        self.payload
    }
}

impl NativeManagedSourceInterpretation {
    pub(crate) fn bind_context(
        self: &Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        factory: Option<&dyn ManagedIndexSourceContextFactory>,
    ) -> Result<(), crate::transactions::data::TransactionCommitError> {
        let Some(factory) = factory else {
            return Ok(());
        };
        let payload = factory
            .bind(execution, self.identity(), self.physical_basis())
            .map_err(|cause| cause.into_commit_error(self.physical_basis()))?;
        // Only the proposal-round constructor calls this, immediately after
        // issue and before applying or lending any Source to a participant.
        self.context.set(BoundSourceContext(payload)).map_err(|_| {
            crate::transactions::data::TransactionCommitError::preparation(
                crate::transactions::data::CommitPreparationError::managed_index_source(
                    self.physical_basis(),
                    super::ManagedIndexSourcePreparationDenial::ForeignSource,
                ),
            )
        })
    }

    pub(crate) fn preparation_metadata(
        self: &Arc<Self>,
    ) -> Option<ManagedIndexSourcePreparationMetadata<'_>> {
        self.context
            .get()
            .map(|bound| ManagedIndexSourcePreparationMetadata {
                identity: self.identity(),
                payload: &bound.0,
            })
    }
}
