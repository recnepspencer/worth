//! Retained lineage source evidence may outlive a failed index registration.

use super::*;

#[cfg(test)]
mod tests;

impl FullVerificationImage<'_> {
    pub(super) fn root_source_is_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: EvidenceView<'_>,
    ) -> Result<bool, FullVerificationStop> {
        let origin = source.selected_native_root();

        if origin.runtime_instance_id() != self.selected.runtime_instance_id()
            || origin.branch_id() != self.selected.branch_id()
            || origin.version_id() > self.selected.version_id()
            || origin.position() > self.selected.position()
        {
            return Err(FullVerificationStop::Foreign);
        }
        for fact in source.source_facts() {
            if !fact_is_current(execution, fact, self.runtime, self.snapshot)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn require_complete_root_upstream(
        &self,
        source: EvidenceView<'_>,
        terminal: &Arc<RecordedSettlementIdentity>,
    ) -> Result<(), FullVerificationStop> {
        let mut actual = OrdSet::new();

        for upstream in source.upstream() {
            actual.insert(Arc::clone(upstream.identity()));
        }

        let row = self
            .state
            .settlements
            .get(terminal)
            .ok_or(FullVerificationStop::OutputEvidenceUnavailable)?;

        if actual.len() != row.consumed_upstream.len() {
            return Err(FullVerificationStop::OutputEvidenceUnavailable);
        }
        for upstream in &row.consumed_upstream {
            if !actual.contains(upstream) {
                return Err(FullVerificationStop::OutputEvidenceUnavailable);
            }
        }
        Ok(())
    }
}
