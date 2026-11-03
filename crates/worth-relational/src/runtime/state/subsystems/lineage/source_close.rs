//! Original lineage aggregate slots, never page counts, admit source retirement.
use super::LineageSubsystem;
use crate::durability::medium::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial, RelationalDurableColdSource,
    RelationalDurableMemberRetirementObservation as Observation,
    RelationalDurableMemberRetirementRefusal as Refusal,
};
use std::sync::Arc;
impl LineageSubsystem {
    pub(crate) fn preflight_node_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Denial> {
        let target = {
            let mut slot = self.node_residency.try_write().ok_or(Denial::Busy)?;
            match slot.as_mut() {
                None => None,
                Some(original) => {
                    // Exported sources retain this aggregate. Inert admitted
                    // node/page metadata retains individual page allocations.
                    Arc::get_mut(original).ok_or(Denial::Busy)?;
                    Some(original.clone())
                }
            }
        };
        if let Some(target) = target {
            target.preflight_source_retirement(source)?;
        }
        Ok(())
    }
    pub(crate) fn loan_node_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        let slot = self.node_residency.try_read().ok_or(Denial::Busy)?;
        match slot.as_ref() {
            Some(original) => original.loan_source_retirement(handoff),
            None => Ok(true),
        }
    }
    pub(crate) fn return_node_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        let slot = self.node_residency.try_read().ok_or(Denial::Busy)?;
        slot.as_ref()
            .ok_or(Denial::Refused(Refusal::ForeignCustody))?
            .return_source_retirement(handoff, source)
    }

    pub(crate) fn preflight_event_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Denial> {
        let target = {
            let mut slot = self.event_residency.try_write().ok_or(Denial::Busy)?;
            match slot.as_mut() {
                None => None,
                Some(original) => {
                    // Exported sources retain this aggregate. Inert admitted
                    // node/page metadata retains individual page allocations.
                    Arc::get_mut(original).ok_or(Denial::Busy)?;
                    Some(original.clone())
                }
            }
        };
        if let Some(target) = target {
            target.preflight_source_retirement(source)?;
        }
        Ok(())
    }
    pub(crate) fn loan_event_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        let slot = self.event_residency.try_read().ok_or(Denial::Busy)?;
        match slot.as_ref() {
            Some(original) => original.loan_source_retirement(handoff),
            None => Ok(true),
        }
    }
    pub(crate) fn return_event_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        let slot = self.event_residency.try_read().ok_or(Denial::Busy)?;
        slot.as_ref()
            .ok_or(Denial::Refused(Refusal::ForeignCustody))?
            .return_source_retirement(handoff, source)
    }
}
