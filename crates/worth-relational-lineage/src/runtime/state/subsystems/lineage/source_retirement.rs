//! Retire each original aggregate registration even when its pages are empty.
use std::sync::{Arc, Mutex};
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableMemberLocator;
use worth_relational_storage::internal_port::durability::medium::member_custody::native_source_slot::NativeSourceMemberState;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::durability::medium::member_custody::native_slot::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial as Denial,
};
use worth_relational_storage::internal_port::durability::medium::member_custody::retirement::RelationalDurableMemberRetirementObservation as Observation;

#[derive(Debug)]
pub(super) struct LineagePageSource(Mutex<NativeSourceMemberState>);

impl LineagePageSource {
    pub(super) fn retain(
        source: &Arc<dyn RelationalDurableColdSource>,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<Self, Cause> {
        let placement = source.retain_members(members)?;
        Ok(Self(Mutex::new(NativeSourceMemberState::new(source.clone(), placement))))
    }

    pub(in crate::internal_port::runtime::state::subsystems::lineage) fn source(
        &self,
    ) -> Result<Arc<dyn RelationalDurableColdSource>, Cause> {
        self.0
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
            .0
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
        self.0
            .try_lock()
            .map_err(|_| Denial::Busy)?
            .loan(handoff)
    }
    pub(in crate::internal_port::runtime) fn return_source_retirement(
        &self,
        handoff: &mut Option<NativeMemberRetirementHandoff>,
        source: &dyn RelationalDurableColdSource,
    ) -> Result<Observation, Denial> {
        self.0
            .try_lock()
            .map_err(|_| Denial::Busy)?
            .return_handoff(handoff, source)
    }
}
