//! A producer decision on a coupled candidate reads the names the envelope's
//! earlier members give their records (#426) from the lent assembly, whose
//! naming authority previews them once per pin.
use super::*;
use worth_relational::facade::runtime::{
    CustomInvariantNameReadDenial as Denial, CustomInvariantNameSource,
    RelationalAppliedCandidateReadIdentity,
};

impl<S: ApplicationSchema> SourceQueryReaderAccess<'_, S> {
    /// Runs `read` over the names the lent assembly's envelope gives its
    /// records, when its current pin is exactly `candidate`; `Unavailable`
    /// otherwise.
    ///
    /// Complexity: as `RelationalAuthoredCoupledCandidate::with_candidate_names`.
    pub fn with_applied_candidate_names(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &RelationalAppliedCandidateReadIdentity,
        read: &mut dyn FnMut(&dyn CustomInvariantNameSource),
    ) -> Result<(), Denial> {
        let assembly = self.applied_managed_sources.ok_or(Denial::Unavailable)?;
        // Composing the plans refuses a naming plan with its exact denial;
        // any other stop only spends the request's work and memory.
        assembly
            .with_candidate_names(execution, candidate, |names| read(names))
            .map_err(|stop| match stop.lineage_name_denial_ref() {
                Some(denial) => Denial::Naming(denial.clone()),
                None => Denial::WorkExhausted,
            })?
            .ok_or(Denial::Unavailable)
    }
}
