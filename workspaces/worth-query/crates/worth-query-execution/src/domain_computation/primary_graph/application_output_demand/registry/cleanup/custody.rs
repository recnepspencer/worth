//! A queued global custody prune pays its complete quote before its first edit.
use super::*;
impl DemandRegistryState {
    pub(in super::super) fn drain_completed_custody(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if !self.pending_custody_prune {
            return Ok(());
        }
        super::super::lifecycle::cleanup_price::price_cleanup(
            self,
            admission,
            Vec::new(),
            &[],
            super::super::refreshed_rejoin::WalkKind::TerminalCleanup,
        )?
        .complete(|priced| {
            priced.prune_completed_custody();
            Ok(())
        });
        Ok(())
    }
}
