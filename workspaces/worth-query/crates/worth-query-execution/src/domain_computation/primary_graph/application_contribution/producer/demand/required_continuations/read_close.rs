//! Retry the original typed successor prefix before releasing its funded backing.
use super::*;
impl<S: ApplicationSchema + 'static> RequiredContinuations<S> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn close_read_request(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), super::super::WorthQueryOutputReadCloseDenial> {
        for entry in &mut self.entries {
            entry.successor.close_read_request(phase, runtime, branch)?;
        }
        // Keep entries, requested claims and funding until the containing demand
        // closes. Refusal never resets an unresolved suffix or a released prefix.
        Ok(())
    }
}
