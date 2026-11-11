//! Readmitted World facts can describe an original, never recreate its delivery.
use super::*;
use crate::domain_computation::primary_graph::provider::WorthQueryCommittedDispatchOutboxReadDenial as Denial;
use worth_runtime_world::facade::{
    CompositeCommitProvenance, CompositeRuntimeWorldCommit, WorldReadmittedPublication,
};

impl WorthQueryCommittedProductPublication {
    pub(in crate::domain_computation::primary_graph) fn from_readmitted(
        original: &WorldReadmittedPublication,
        commit: &CompositeRuntimeWorldCommit,
    ) -> Result<Self, Denial> {
        let Some(native) = commit.relational_publication_identity() else {
            return Err(Denial::CommitMismatch);
        };
        if commit.identity() != original.after().selected_commit()
            || !matches!(commit.provenance(), CompositeCommitProvenance::Publication(attempt) if attempt == original.attempt_identity())
            || native.commit_id() != original.native_receipt().commit_id
            || native.version_id() != original.native_receipt().version_id
            || native.authoring_branch() != &original.native_receipt().branch_id
            || original.before().branch_identity() != original.after().branch_identity()
            || original.before().lifecycle_incarnation() != original.after().lifecycle_incarnation()
            || commit.relational_change() != CompositeComponentChangePosture::Published
        {
            return Err(Denial::CommitMismatch);
        }
        Ok(Self {
            custody: PublicationCustody::Described(Arc::new(DescriptiveProductPublication {
                product_branch: original.after().branch_identity().clone(),
                product_incarnation: original.after().lifecycle_incarnation(),
                product_generation: original.after().reference_generation(),
                composite_commit: commit.identity().clone(),
                publication_attempt: original.attempt_identity().clone(),
                relational_commit: original.native_receipt().clone(),
                relational_posture: commit.relational_change(),
                signal_posture: commit.signal_change(),
                signal_publication: commit.signal_publication_identity().cloned(),
                // A prior live conditional counter is not recovered source evidence.
                conditional_definition_generation: None,
            })),
        })
    }
}
