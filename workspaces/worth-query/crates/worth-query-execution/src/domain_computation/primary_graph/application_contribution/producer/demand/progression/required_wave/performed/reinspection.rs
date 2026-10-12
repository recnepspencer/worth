//! Only another member's actual publication pays for a queued inspection.
use super::*;
#[derive(Clone, Copy)]
pub(in super::super) struct InspectionStamp(u64);
/// Move-only permission. No caller failure arm accepts this capability.
pub(in super::super) struct ReinspectionCredit {
    _publication: InspectionStamp,
}
impl PerformedMembers {
    pub(in super::super) fn inspection(&self) -> InspectionStamp {
        InspectionStamp(self.commit_generation)
    }
    pub(in super::super) fn reinspection(
        &self,
        key: &WorthQueryOutputDemandKey,
        inspected: InspectionStamp,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<ReinspectionCredit>, WorthQueryOutputDemandDenial> {
        for entry in &self.entries {
            admission
                .charge_external_work(entry.comparison_work()?)
                .map_err(admission_denial)?;
            if entry.published_at > inspected.0 && !entry.names(key) {
                return Ok(Some(ReinspectionCredit {
                    _publication: self.inspection(),
                }));
            }
        }
        Ok(None)
    }
}
#[cfg(test)]
mod tests;
