//! Select the native hint for the already admitted Product branch.

use worth_relational::facade::history::BranchId;

use super::*;

impl SelectedRequiredWork {
    /// A member can retain prepared attempts for several native cells. This
    /// rotates only a matching hint to the acknowledged head; a foreign hint
    /// stays queued and cannot authorize the selected Product.
    pub(super) fn for_selected_branch(
        &mut self,
        branch: &BranchId,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(4)
            .map_err(|_| work_denial())?;
        let mut state = self
            .membership
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.active
            || !state.marked
            || state.version_exhausted
            || state.version != self.version
        {
            return Ok(false);
        }
        let per_hint = branch.0.len().checked_add(4).ok_or_else(work_denial)?;
        let per_hint = u64::try_from(per_hint).map_err(|_| work_denial())?;

        let next_version = state.version.checked_add(1);
        let mut cursor = &mut state.native_hints;
        let mut at_head = true;
        loop {
            let Some(hint) = cursor.as_ref() else {
                break;
            };
            admission
                .charge_external_work(per_hint)
                .map_err(|_| work_denial())?;
            if hint.branch.branch_id() == branch {
                break;
            }
            cursor = &mut cursor.as_mut().expect("visited hint remains present").next;
            at_head = false;
        }
        if cursor.is_some() {
            if !at_head {
                let next_version = next_version.ok_or_else(|| {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::SchedulingRejected,
                        "selected required-work version is exhausted",
                    )
                })?;
                let mut matching = cursor.take().expect("matching native hint exists");
                *cursor = matching.next.take();
                matching.next = state.native_hints.take();
                state.native_hints = Some(matching);
                state.version = next_version;
                self.version = state.version;
            }
            let hint = state.native_hints.as_ref().expect("matching head retained");
            self.kind = SelectedRequiredWorkKind::Native {
                observer: hint.observer.clone(),
                branch: Arc::clone(&hint.branch),
            };
            return Ok(true);
        }
        // Other-cell native attempts remain untouched. Independent local and
        // initial causes for this record may still advance.
        self.kind = if let Some(settlement) = state.local_settlement.as_ref() {
            SelectedRequiredWorkKind::Local {
                settlement: Arc::clone(settlement),
            }
        } else if state.discontinuity_pending {
            SelectedRequiredWorkKind::Discontinuity
        } else if state.unresolved_initial {
            SelectedRequiredWorkKind::UnresolvedInitial
        } else {
            return Ok(false);
        };
        Ok(true)
    }
}

fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "selected native-hint comparison exceeds request work",
    )
}
