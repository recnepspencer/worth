//! Only another member's actual publication pays for a queued inspection.
use super::*;
#[derive(Clone, Copy)]
pub struct InspectionStamp(u64);
/// Move-only permission. No caller failure arm accepts this capability.
pub struct ReinspectionCredit {
    _publication: InspectionStamp,
}
impl PerformedMembers {
    pub fn inspection(&self) -> InspectionStamp {
        InspectionStamp(self.commit_generation)
    }
    pub fn reinspection(
        &self,
        key: &WorthQueryOutputDemandKey,
        inspected: InspectionStamp,
    ) -> Result<Option<ReinspectionCredit>, WorthQueryOutputDemandDenial> {
        for entry in &self.entries {
            if entry.published_at > inspected.0 && !entry.names(key) {
                return Ok(Some(ReinspectionCredit {
                    _publication: self.inspection(),
                }));
            }
        }
        Ok(None)
    }
}
