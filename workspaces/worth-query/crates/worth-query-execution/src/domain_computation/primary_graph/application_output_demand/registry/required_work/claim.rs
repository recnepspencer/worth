//! A queue pop claims a physical row, without selecting its current cause.
use super::*;
pub(in crate::domain_computation::primary_graph) struct RequiredWorkClaim {
    pub(super) membership: Arc<RequiredWorkMembership>,
    pub(super) armed: bool,
}
impl RequiredWorkClaim {
    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &WorthQueryOutputDemandKey {
        self.membership.key()
    }
    pub(in crate::domain_computation::primary_graph) fn same_record(
        &self,
        ready: &SelectedReadyReadmission,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(1)
            .map_err(|_| selection::work_denial())?;
        Ok(Arc::ptr_eq(&self.membership, &ready.membership))
    }
    /// Owner tests exercise overlapping selected versions outside an advance.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn select_without_advance(
        mut self,
    ) -> Option<SelectedRequiredWork> {
        let selected = self.select();
        self.membership
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .claimed = false;
        self.armed = false;
        selected
    }
    /// Only inspection selects the live cause and version. An unfinished claim
    /// or selection requeues on drop; a pop itself acknowledges nothing.
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn select(
        &mut self,
    ) -> Option<SelectedRequiredWork> {
        let member = self
            .membership
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !member.active || !member.marked || member.version_exhausted {
            return None;
        }
        let kind = if let Some(hint) = member.native_hints.as_ref() {
            SelectedRequiredWorkKind::Native {
                observer: hint.observer.clone(),
                branch: Arc::clone(&hint.branch),
            }
        } else if let Some(settlement) = member.local_settlement.as_ref() {
            SelectedRequiredWorkKind::Local {
                settlement: Arc::clone(settlement),
            }
        } else if member.discontinuity_pending {
            SelectedRequiredWorkKind::Discontinuity
        } else {
            assert!(
                member.unresolved_initial,
                "a marked required row has one cause"
            );
            SelectedRequiredWorkKind::UnresolvedInitial
        };
        let selected = SelectedRequiredWork {
            membership: Arc::clone(&self.membership),
            certification_membership: Arc::clone(&self.membership),
            version: member.version,
            kind,
            armed: true,
        };
        drop(member);
        Some(selected)
    }
}
impl Drop for RequiredWorkClaim {
    fn drop(&mut self) {
        if self.armed {
            self.membership
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .claimed = false;
            if let Some(queue) = self.membership.queue.upgrade() {
                queue.enqueue(&self.membership);
            }
        }
    }
}
