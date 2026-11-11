//! One canonical sequence reservation owner for ordinary and applied proposals.
use crate::{
    history::data::CommitId, identity::data::VersionId, mvcc::RelationalTransactionValidationInput,
    runtime::RelationalPreparationRuntime, transactions::data::TransactionCommitError,
};
impl RelationalPreparationRuntime {
    pub(super) fn reserve_proposal_sequence(
        &self,
        validation_input: &RelationalTransactionValidationInput,
    ) -> Result<(VersionId, u64, CommitId), TransactionCommitError> {
        let proposed_version_id = match validation_input.coupled_proposal() {
            Some(prior) => prior.proposed_version_id(),
            None => self.history.reserve_version_id().ok_or_else(|| {
                crate::transactions::data::TransactionCommitError::publication(
                    crate::publication::data::PublicationError::new(
                        crate::publication::bundle::PublicationStage::BundleAssembly,
                        "canonical version sequence overflow",
                    ),
                )
            })?,
        };
        let ordinal = match validation_input.coupled_proposal() {
            Some(prior) => prior.ordinal(),
            None => self.services.next_proposal_ordinal().ok_or_else(|| {
                crate::transactions::data::TransactionCommitError::preparation(
                    crate::transactions::data::CommitPreparationError::proposal_identity_exhausted(
                        validation_input.target_branch().clone(),
                        proposed_version_id,
                    ),
                )
            })?,
        };
        let commit_id = match validation_input.coupled_proposal() {
            Some(prior) => prior.commit_id(),
            None => self.history.reserve_commit_id().ok_or_else(|| {
                crate::transactions::data::TransactionCommitError::publication(
                    crate::publication::data::PublicationError::new(
                        crate::publication::bundle::PublicationStage::BundleAssembly,
                        "canonical commit identity capacity exhausted",
                    ),
                )
            })?,
        };
        Ok((proposed_version_id, ordinal, commit_id))
    }
}
