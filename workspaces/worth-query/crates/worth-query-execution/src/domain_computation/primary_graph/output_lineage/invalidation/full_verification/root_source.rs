//! Retained lineage source evidence may outlive a failed index registration.

use super::*;

#[cfg(test)]
mod tests;

impl FullVerificationImage<'_> {
    pub(super) fn root_source_is_current(
        &self,
        source: EvidenceView<'_>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, FullVerificationStop> {
        let origin = source.selected_native_root();
        admission.work(4 + origin.branch_id().0.len() as u64)?;
        if origin.runtime_instance_id() != self.selected.runtime_instance_id()
            || origin.branch_id() != self.selected.branch_id()
            || origin.version_id() > self.selected.version_id()
            || origin.position() > self.selected.position()
        {
            return Err(FullVerificationStop::Alignment(
                FullVerificationReason::ForeignSource,
            ));
        }
        for fact in source.source_facts() {
            if !fact_is_current(fact, self.runtime, self.snapshot, admission)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn require_complete_root_upstream(
        &self,
        source: EvidenceView<'_>,
        terminal: &Arc<RecordedSettlementIdentity>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), FullVerificationStop> {
        let mut actual = OrdSet::new();
        admission.work(source.upstream().len() as u64)?;
        for upstream in source.upstream() {
            admission.admit_visited_settlement(actual.len())?;
            actual.insert(Arc::clone(upstream.identity()));
        }
        admission.ordered_read(self.state.settlements.len())?;
        let row = self
            .state
            .settlements
            .get(terminal)
            .ok_or(FullVerificationStop::OutputEvidenceUnavailable)?;
        admission.work(1)?;
        if actual.len() != row.consumed_upstream.len() {
            return Err(FullVerificationStop::OutputEvidenceUnavailable);
        }
        for upstream in &row.consumed_upstream {
            admission.work(1)?;
            admission.ordered_read(actual.len())?;
            if !actual.contains(upstream) {
                return Err(FullVerificationStop::OutputEvidenceUnavailable);
            }
        }
        Ok(())
    }
}
