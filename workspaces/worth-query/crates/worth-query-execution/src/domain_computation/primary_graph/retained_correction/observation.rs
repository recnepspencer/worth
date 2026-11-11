//! Borrowing a current receiver yields retained DATA, never its ports or pins.
use super::catalog::MaterialBody;
use super::record::{MaterialCapture, MaterialReady};
use super::*;

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct RetainedCorrectionMaterial {
    _funding: Arc<MaterialFunding>,
    pub(in crate::domain_computation::primary_graph) capture: Arc<MaterialCapture>,
    pub(in crate::domain_computation::primary_graph) ready: Arc<MaterialReady>,
}
impl CorrectionMaterialOwner {
    pub(in crate::domain_computation::primary_graph) fn admit_original_metadata(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<(), Denial> {
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        let mut matches = state
            .catalog
            .material
            .values()
            .filter(|entry| match &entry.body {
                MaterialBody::Ready { ready, .. } => &ready.commit == commit,
                MaterialBody::Expired { commit: expired } => expired == commit,
                MaterialBody::Captured(_) | MaterialBody::Abandoned => false,
            });
        if matches.next().is_none() {
            return Err(Denial::Unavailable);
        }
        if matches.next().is_some() {
            return Err(Denial::OriginalMismatch);
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn observe_original(
        &self,
        commit: &worth_relational::facade::history::RelationalCommitReceipt,
    ) -> Result<RetainedCorrectionMaterial, Denial> {
        let state = self.state();
        if state.pending.is_some() {
            return Err(Denial::PublicationPending);
        }
        let mut found = None;
        for entry in state.catalog.material.values() {
            match &entry.body {
                MaterialBody::Ready { capture, ready } if &ready.commit == commit => {
                    if found.is_some() {
                        return Err(Denial::OriginalMismatch);
                    }
                    found = Some(RetainedCorrectionMaterial {
                        _funding: Arc::clone(&state.funding),
                        capture: Arc::clone(capture),
                        ready: Arc::clone(ready),
                    });
                }
                MaterialBody::Expired { commit: expired } if expired == commit => {
                    return Err(Denial::RetainedMaterialExpired)
                }
                MaterialBody::Captured(_)
                | MaterialBody::Ready { .. }
                | MaterialBody::Expired { .. }
                | MaterialBody::Abandoned => {}
            }
        }
        found.ok_or(Denial::Unavailable)
    }
}
