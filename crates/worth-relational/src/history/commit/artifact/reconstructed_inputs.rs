//! Source reconstruction replaces one catalog DATA object while sharing its exact cell.
use super::*;
impl RelationalCommitArtifact {
    pub(crate) fn with_reconstructed_original_inputs(
        &self,
        inputs: OriginalCanonicalInputs,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        use crate::execution::RelationalExecutionDenialCause as Cause;
        let control = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .0
            .pad_to_align()
            .size();
        let parents = std::alloc::Layout::array::<CommitId>(self.parentage.as_slice().len())
            .map_err(|_| Cause::ChargedBytesOverflow)?
            .size();
        let bytes = control
            .checked_add(parents)
            .and_then(|n| n.checked_add(self.identity.authoring_branch().0.len()))
            .ok_or(Cause::ChargedBytesOverflow)?;
        let allocation_memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(Cause::from)?;
        request.check_live().map_err(Cause::from)?;
        Ok(Self {
            identity: self.identity.clone(),
            parentage: self.parentage.clone(),
            roots: self.roots.clone(),
            schema_version: self.schema_version,
            envelope: self.envelope.clone(),
            original_inputs: Some(inputs),
            allocation_memory,
        })
    }
}
