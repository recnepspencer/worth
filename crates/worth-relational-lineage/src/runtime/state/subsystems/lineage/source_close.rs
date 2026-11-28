//! Original lineage aggregate slots, never page counts, admit source retirement.
use super::LineageSubsystem;
use crate::storage::durability::medium::cold_source::RelationalDurableColdSource;
use crate::storage::durability::medium::member_custody::native_slot::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial,
};
use crate::storage::durability::medium::member_custody::retirement::{
    RelationalDurableMemberRetirementObservation as Observation,
    RelationalDurableMemberRetirementRefusal as Refusal,
};
use std::sync::Arc;
impl LineageSubsystem {
    pub fn preflight_node_source_retirement(
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
            target.custody.preflight_source_retirement(source)?;
        }
        Ok(())
    }
    pub fn loan_node_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        let slot = self.node_residency.try_read().ok_or(Denial::Busy)?;
        match slot.as_ref() {
            Some(original) => original.custody.loan_source_retirement(handoff),
            None => Ok(true),
        }
    }
    pub fn return_node_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        let slot = self.node_residency.try_read().ok_or(Denial::Busy)?;
        slot.as_ref()
            .ok_or(Denial::Refused(Refusal::ForeignCustody))?
            .custody.return_source_retirement(handoff, source)
    }

    pub fn preflight_event_source_retirement(
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
            target.custody.preflight_source_retirement(source)?;
        }
        Ok(())
    }
    pub fn loan_event_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        let slot = self.event_residency.try_read().ok_or(Denial::Busy)?;
        match slot.as_ref() {
            Some(original) => original.custody.loan_source_retirement(handoff),
            None => Ok(true),
        }
    }
    pub fn return_event_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        let slot = self.event_residency.try_read().ok_or(Denial::Busy)?;
        slot.as_ref()
            .ok_or(Denial::Refused(Refusal::ForeignCustody))?
            .custody.return_source_retirement(handoff, source)
    }
}
