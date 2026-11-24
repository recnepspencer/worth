//! Retire each original aggregate registration even when its pages are empty.
use super::{
    event_residency::RuntimeLineageEventResidency, node_residency::RuntimeLineageNodeResidency,
};
use std::sync::Arc;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::durability::medium::member_custody::native_slot::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial,
};
use worth_relational_storage::internal_port::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementObservation as Observation;

impl RuntimeLineageNodeResidency {
    pub(in crate::internal_port::runtime::state::subsystems::lineage) fn source(
        &self,
    ) -> Result<Arc<dyn RelationalDurableColdSource>, Cause> {
        self.custody
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .source()
            .cloned()
            .ok_or(Cause::SourceUnavailable)
    }
    pub(in crate::internal_port::runtime) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Denial> {
        let token = self
            .custody
            .try_lock()
            .map_err(|_| Denial::Busy)?
            .preflight_snapshot(source)?;
        if let Some(token) = token {
            token
                .preflight_retirement(source)
                .map_err(Denial::Refused)?;
        }
        Ok(())
    }
    pub(in crate::internal_port::runtime) fn loan_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        self.custody
            .try_lock()
            .map_err(|_| Denial::Busy)?
            .loan(handoff)
    }
    pub(in crate::internal_port::runtime) fn return_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        self.custody
            .try_lock()
            .map_err(|_| Denial::Busy)?
            .return_handoff(handoff, source)
    }
}

impl RuntimeLineageEventResidency {
    pub(in crate::internal_port::runtime::state::subsystems::lineage) fn source(
        &self,
    ) -> Result<Arc<dyn RelationalDurableColdSource>, Cause> {
        self.custody
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .source()
            .cloned()
            .ok_or(Cause::SourceUnavailable)
    }
    pub(in crate::internal_port::runtime) fn preflight_source_retirement(
        &self,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<(), Denial> {
        let token = self
            .custody
            .try_lock()
            .map_err(|_| Denial::Busy)?
            .preflight_snapshot(source)?;
        if let Some(token) = token {
            token
                .preflight_retirement(source)
                .map_err(Denial::Refused)?;
        }
        Ok(())
    }
    pub(in crate::internal_port::runtime) fn loan_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
    ) -> Result<bool, Denial> {
        self.custody
            .try_lock()
            .map_err(|_| Denial::Busy)?
            .loan(handoff)
    }
    pub(in crate::internal_port::runtime) fn return_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        self.custody
            .try_lock()
            .map_err(|_| Denial::Busy)?
            .return_handoff(handoff, source)
    }
}
