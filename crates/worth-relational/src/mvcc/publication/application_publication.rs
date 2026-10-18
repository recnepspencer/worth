use crate::transactions::data::{CreatedEntityRef, TransactionCommitError};

#[cfg(test)]
mod absent_optional_field;
#[cfg(test)]
mod tests;

/// Native installation binding for one exact application publisher occurrence.
/// Descriptive producer names cannot construct or substitute this binding.
#[derive(Clone, Debug)]
pub struct RelationalInstalledApplicationPublicationAuthority {
    pub(crate) runtime_instance_id: u64,
    pub(crate) custom_invariant_generation: u64,
    pub(crate) schema_digest: [u8; 32],
    pub(crate) authority: worth_proof::InstalledApplicationPublicationAuthority,
}

#[derive(Debug)]
pub(crate) struct RequiredOutputPublicationAdmission {
    pub(crate) installed: RelationalInstalledApplicationPublicationAuthority,
    pub(crate) proof: worth_proof::AdmittedRequiredOutputPublication,
    pub(crate) targets: Vec<CreatedEntityRef>,
}

impl RelationalInstalledApplicationPublicationAuthority {
    pub(crate) fn admits_checkpoint_program_activation(
        &self,
        proof: &worth_proof::AdmittedCheckpointProgramActivation,
    ) -> bool {
        self.authority.admits_checkpoint_program_activation(proof)
    }
}

impl crate::mvcc::ValidatedRelationalProposal {
    /// Bind a running installed producer before immutable publication preparation.
    /// Target references remain untrusted selectors until native allocation and
    /// canonical operation/payload coverage have been checked.
    pub fn bind_required_output_publication(
        mut self,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        proof: worth_proof::AdmittedRequiredOutputPublication,
        targets: Vec<CreatedEntityRef>,
    ) -> Result<Self, TransactionCommitError> {
        if self.required_output_publication.is_some()
            || !installed.authority.admits_required_output(&proof)
            || self
                .validation_input
                .basis()
                .descriptor()
                .runtime_instance_id()
                != installed.runtime_instance_id
            || self.custom_invariant_generation != installed.custom_invariant_generation
        {
            return Err(origin_denial(
                "required-output publication has a foreign or repeated installed owner",
            ));
        }
        self.required_output_publication = Some(RequiredOutputPublicationAdmission {
            installed: installed.clone(),
            proof,
            targets,
        });
        Ok(self)
    }
}

pub(crate) fn origin_denial(detail: impl Into<String>) -> TransactionCommitError {
    TransactionCommitError::publication(crate::publication::data::PublicationError::new(
        crate::publication::bundle::PublicationStage::BundleAssembly,
        detail,
    ))
}
