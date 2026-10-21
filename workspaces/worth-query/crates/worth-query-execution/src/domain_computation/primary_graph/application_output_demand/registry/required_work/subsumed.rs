//! One Current proof on the selected branch discharges every native hint it
//! covers for that member, not one hint per advance.

use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::CompanionPublicationCompletion;

use super::*;
use crate::domain_computation::primary_graph::application_contribution::CurrentProof;

impl SelectedRequiredWork {
    /// Stamp the member's same-branch hints whose publication has already
    /// settled. The caller then checks that the head is still the wave's
    /// position: every stamped publication settled before that check, so the
    /// wave's Current proof covers it. A hint still Prepared here is not
    /// stamped, even if it installs later.
    pub(in crate::domain_computation::primary_graph) fn stamp_settled_hints(
        &self,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let SelectedRequiredWorkKind::Native { branch, .. } = &self.kind else {
            return Ok(());
        };
        let branch = branch.branch_id();
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
            if hint.branch.branch_id() == branch
                && hint.observer.state() != CompanionPublicationCompletion::Prepared
            {
                hint.seen_settled_at = Some(self.version);
            }
            cursor = hint.next.as_deref_mut();
        }
        Ok(())
    }

    /// A Current inspection covers this row's selected local/initial causes as
    /// well as its settled same-branch hints. Newer marks remain unacknowledged.
    pub(in crate::domain_computation::primary_graph) fn acknowledge_current_admitted<T>(
        self,
        proof: CurrentProof<T>,
    ) -> Result<Option<ReplacedRequiredWorkHint>, WorthQueryOutputDemandDenial> {
        if !proof.certifies_record(&self.certification_membership) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "Current proof does not certify this retained record and basis",
            ));
        }

        let extra = {
            let mut state = self
                .membership
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.version == self.version && !state.version_exhausted {
                match self.kind {
                    SelectedRequiredWorkKind::Native { .. } => {
                        state.unresolved_initial = false;
                        state.local_settlement.take()
                    }
                    SelectedRequiredWorkKind::Local { .. } => {
                        state.unresolved_initial = false;
                        None
                    }
                    SelectedRequiredWorkKind::UnresolvedInitial
                    | SelectedRequiredWorkKind::Discontinuity => state.local_settlement.take(),
                }
            } else {
                None
            }
        };
        let result = self.acknowledge_subsumed();
        drop(extra);
        result
    }

    fn acknowledge_subsumed(
        mut self,
    ) -> Result<Option<ReplacedRequiredWorkHint>, WorthQueryOutputDemandDenial> {
        let SelectedRequiredWorkKind::Native { branch, .. } = &self.kind else {
            return Ok(self.consume_selected_cause());
        };
        let branch = Arc::clone(branch);
        let cleared = self
            .membership
            .acknowledge_branch(self.version, branch.branch_id());
        self.armed = cleared.as_ref().is_none_or(|(_, more)| *more);
        Ok(cleared.map(|(removed, _)| removed))
    }
}

impl RequiredWorkMembership {
    fn acknowledge_branch(
        &self,
        version: u64,
        branch: &BranchId,
    ) -> Option<(ReplacedRequiredWorkHint, bool)> {
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
