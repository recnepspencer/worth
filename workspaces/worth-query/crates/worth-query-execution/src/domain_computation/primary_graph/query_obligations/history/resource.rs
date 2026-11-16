//! Resource publication uses the same serialized history and pending original.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::OwnedAsyncObligationIdentity;
/// Progress of the original live owner's logical disposal, not a release proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub(in crate::domain_computation::primary_graph) enum ResourceDisposal {
    #[default]
    Retained,
    Discharging,
    Discharged,
}
impl ResourceDisposal {
    fn resume_after_settlement(&mut self, retained: bool) -> Result<(), QueryObligationDenial> {
        // Only the original disposal-in-progress can consume a missing record
        // as acknowledgment. An acquire's no-effect is different.
        match (*self, retained) {
            (Self::Discharged, _) => {}
            (_, true) => *self = Self::Discharging,
            (Self::Discharging, false) => *self = Self::Discharged,
            (Self::Retained, false) => return Err(QueryObligationDenial::UnknownObligation),
        }
        Ok(())
    }
}
impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn retain_resource(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: QueryResourceOriginal,
    ) -> Result<
        Option<(OwnedAsyncObligationIdentity, Option<QueryObligationDenial>)>,
        QueryObligationDenial,
    > {
        let Some(medium) = self.medium.get() else {
            return Ok(None);
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let identity = {
            let mut state = self.state();
            let journal = state
                .journal
                .as_mut()
                .ok_or(QueryObligationDenial::SupportUnavailable)?;
            let position = journal
                .ledger
                .position()
                .checked_add(1)
                .and_then(std::num::NonZeroU64::new)
                .ok_or(QueryObligationDenial::IdentityExhausted)?;
            let identity = OwnedAsyncObligationIdentity::from_recorded(medium.home(), position);
            journal.prepare_pending(QueryResourceRecord::Retain(QueryResourceObligation {
                identity,
                original,
                handoff: QueryResourceHandoff::default(),
            }))?;
            identity
        };
        let pending = driver.finish_pending(medium.as_ref(), request).err();
        Ok(Some((identity, pending)))
    }
    pub(in crate::domain_computation::primary_graph) fn resource_retained(
        &self,
        identity: OwnedAsyncObligationIdentity,
    ) -> bool {
        self.state()
            .journal
            .as_ref()
            .is_some_and(|journal| journal.ledger.resources.contains_key(&identity))
    }
    pub(in crate::domain_computation::primary_graph) fn resource_obligation(
        &self,
        identity: OwnedAsyncObligationIdentity,
    ) -> Option<QueryResourceObligation> {
        self.state()
            .journal
            .as_ref()?
            .ledger
            .resources
            .get(&identity)
            .cloned()
    }
    pub(in crate::domain_computation::primary_graph) fn settle_resource_publication(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        if let Some(medium) = self.medium.get() {
            self.lock_publication()?
                .finish_pending(medium.as_ref(), request)?;
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn record_resource_handoff(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: OwnedAsyncObligationIdentity,
        handoff: QueryResourceHandoff,
    ) -> Result<(), QueryObligationDenial> {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if self
            .state()
            .journal
            .as_ref()
            .and_then(|journal| journal.ledger.resources.get(&identity))
            .is_some_and(|row| row.handoff == handoff)
        {
            return Ok(());
        }
        driver.publish(
            medium.as_ref(),
            QueryResourceRecord::Handoff { identity, handoff },
            request,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn discharge_resource(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: OwnedAsyncObligationIdentity,
    ) -> Result<(), QueryObligationDenial> {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        driver.publish(
            medium.as_ref(),
            QueryResourceRecord::Discharge { identity },
            request,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn finish_live_resource_disposal(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: OwnedAsyncObligationIdentity,
        disposal: &mut ResourceDisposal,
    ) -> Result<(), QueryObligationDenial> {
        if *disposal == ResourceDisposal::Discharged {
            return Ok(());
        }
        self.settle_resource_publication(request)?;
        disposal.resume_after_settlement(self.resource_retained(identity))?;
        if *disposal == ResourceDisposal::Discharged {
            return Ok(());
        }
        self.discharge_resource(request, identity)?;
        *disposal = ResourceDisposal::Discharged;
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn preflight_resource_close(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryObligationDenial> {
        self.settle_resource_publication(request)?;
        let state = self.state();
        if let Some(row) = state.journal.as_ref().and_then(|journal| {
            journal
                .ledger
                .resources
                .values()
                .find(|row| row.handoff.release.is_none())
        }) {
            return Err(QueryObligationDenial::OwnedAsyncPending {
                identity: row.identity,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
