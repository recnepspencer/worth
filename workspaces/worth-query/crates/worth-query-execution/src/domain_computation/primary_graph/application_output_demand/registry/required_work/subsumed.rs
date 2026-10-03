//! One Current proof on the selected branch discharges every native hint it
//! covers for that member, not one hint per advance.

use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::CompanionPublicationCompletion;

use super::*;

impl SelectedRequiredWork {
    /// Stamp the member's same-branch hints whose publication has already
    /// settled. The caller then checks that the head is still the wave's
    /// position: every stamped publication settled before that check, so the
    /// wave's Current proof covers it. A hint still Prepared here is not
    /// stamped, even if it installs later.
    pub(in crate::domain_computation::primary_graph) fn stamp_settled_hints(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let SelectedRequiredWorkKind::Native { branch, .. } = &self.kind else {
            return Ok(());
        };
        let branch = branch.branch_id();
        let per_hint = hint_work(branch);
        let mut state = self
            .membership
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.version != self.version || state.version_exhausted {
            return Ok(());
        }
        let mut cursor = state.native_hints.as_deref_mut();
        while let Some(hint) = cursor {
            admission
                .charge_external_work(per_hint)
                .map_err(|_| required_ack_work_denial())?;
            if hint.branch.branch_id() == branch
                && hint.observer.state() != CompanionPublicationCompletion::Prepared
            {
                hint.seen_settled_at = Some(self.version);
            }
            cursor = hint.next.as_deref_mut();
        }
        Ok(())
    }

    /// Acknowledge the selected cause after a Current proof at a head that the
    /// selected hint's publication had already reached. The member's other
    /// hints on that branch are covered too when they were stamped settled at
    /// this selection. Any newer mark moves the version and leaves every hint
    /// in place.
    pub(in crate::domain_computation::primary_graph) fn acknowledge_subsumed_admitted(
        mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<ReplacedRequiredWorkHint>, WorthQueryOutputDemandDenial> {
        let SelectedRequiredWorkKind::Native { branch, .. } = &self.kind else {
            return self.acknowledge_admitted(admission);
        };
        let branch = Arc::clone(branch);
        charge_acknowledgement(admission)?;
        let cleared =
            self.membership
                .acknowledge_branch(self.version, branch.branch_id(), admission);
        self.armed = cleared.as_ref().is_none_or(|(_, more)| *more);
        Ok(cleared.map(|(removed, _)| removed))
    }
}

impl RequiredWorkMembership {
    fn acknowledge_branch(
        &self,
        version: u64,
        branch: &BranchId,
        admission: &mut InvalidationEditAdmission,
    ) -> Option<(ReplacedRequiredWorkHint, bool)> {
        let per_hint = hint_work(branch);
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.version != version || state.version_exhausted {
            return None;
        }
        let mut removed = state
            .native_hints
            .take()
            .expect("selected native hint exists");
        state.native_hints = removed.next.take();
        let mut cursor = &mut state.native_hints;
        loop {
            let covered = match cursor.as_ref() {
                None => break,
                // Coalescing is an optimization of later passes: an exhausted
                // request keeps the remaining hints for its next advance.
                Some(_) if admission.charge_external_work(per_hint).is_err() => break,
                Some(hint) => {
                    hint.branch.branch_id() == branch && hint.seen_settled_at == Some(version)
                }
            };
            if covered {
                let mut node = cursor.take().expect("visited hint remains present");
                *cursor = node.next.take();
                node.next = removed.next.take();
                removed.next = Some(node);
            } else {
                cursor = &mut cursor.as_mut().expect("visited hint remains present").next;
            }
        }
        let more = finish_acknowledgement(&mut state);
        Some((
            ReplacedRequiredWorkHint {
                _native: Some(removed),
                _settlement: None,
            },
            more,
        ))
    }
}

fn hint_work(branch: &BranchId) -> u64 {
    u64::try_from(branch.0.len().saturating_add(4)).unwrap_or(u64::MAX)
}
