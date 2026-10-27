//! Independent Native comparison when no derived mark image covers the read.
use super::*;

impl FullVerificationImage<'_> {
    pub(super) fn verify_native_evidence(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: EvidenceView<'_>,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        let mut pending = Vec::new();
        let mut visited = OrdSet::new();
        reserve_pending(&mut pending, 1)?;
        pending.push(root);
        while let Some(source) = pending.pop() {
            if visited.contains(source.identity()) {
                continue;
            }
            visited.insert(Arc::clone(source.identity()));
            if !self.root_source_is_current(execution, source)? {
                return Ok(FullVerificationDecision::Changed);
            }
            let witness = source
                .native_output_witness()
                .ok_or(FullVerificationStop::OutputEvidenceUnavailable)?;
            if !witness.unchanged_in(self.runtime, self.snapshot, execution)? {
                return Ok(FullVerificationDecision::Changed);
            }
            reserve_pending(&mut pending, source.upstream().len())?;
            pending.extend(source.upstream().iter().map(EvidenceView::from));
        }
        Ok(FullVerificationDecision::Current)
    }
}
