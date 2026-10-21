//! Issued prerequisite rows retained by the last committed publication.
use super::super::super::WorthQueryOutputDemandRegistry;
use super::*;

pub(in crate::domain_computation::primary_graph) struct DeclaredRequiredPredecessors {
    keys: Vec<Arc<WorthQueryOutputDemandKey>>,
}
fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "published prerequisite scheduling exceeds admitted scratch",
    )
}
impl DeclaredRequiredPredecessors {
    /// This schedules an existing required occurrence; it never grants Current
    /// or invents a requirement from a name in the application declaration.
    pub(in crate::domain_computation::primary_graph) fn contains_occurrence(
        &self,
        selected: &WorthQueryOutputDemandKey,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        for key in &self.keys {
            if key.same_occurrence(selected) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
impl WorthQueryOutputDemandRegistry {
    /// Pins the issued prerequisites of a caller that actually reached a
    /// pending consumed edge. These are scheduling pins, never Current proofs.
    pub(in crate::domain_computation::primary_graph) fn declared_required_readmissions(
        &self,
        selected: &SelectedReadyReadmission,
    ) -> Result<Vec<SelectedReadyReadmission>, WorthQueryOutputDemandDenial> {
        let declared = self.declared_required_predecessors(selected.key())?;
        let count = declared.keys.len();

        let mut ready = Vec::new();
        ready
            .try_reserve_exact(count)
            .map_err(|_| capacity_denial())?;
        if ready.capacity() != count {
            return Err(capacity_denial());
        }
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for key in declared.keys {
            let Some(head) =
                super::super::super::required_stop::lineage_head(&state.records, &key)?
            else {
                continue;
            };

            if let Some(member) = super::super::pending_readmission::ready_member(&state, &head)? {
                ready.push(member);
            }
        }
        Ok(ready)
    }
    pub(in crate::domain_computation::primary_graph) fn declared_required_predecessors(
        &self,
        selected: &WorthQueryOutputDemandKey,
    ) -> Result<DeclaredRequiredPredecessors, WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let Some(record) = state.records.get(selected) else {
            return Ok(DeclaredRequiredPredecessors { keys: Vec::new() });
        };
        let count = record.prerequisites.len();

        let mut keys = Vec::new();
        keys.try_reserve_exact(count)
            .map_err(|_| capacity_denial())?;
        if keys.capacity() != count {
            return Err(capacity_denial());
        }
        keys.extend(record.prerequisites.iter().map(Arc::clone));
        Ok(DeclaredRequiredPredecessors { keys })
    }
}

impl WorthQueryOutputDemandRegistry {
    /// The member still owns this exact consumed-output obligation, even when
    /// changed own facts need a fresh decision. Only an already Published
    /// registry frame can be delivered; no new work or claim is created here.
    pub(in crate::domain_computation::primary_graph) fn take_required_published_successor(
        &self,
        selected: &SelectedReadyReadmission,
    ) -> Result<Option<super::super::super::RequiredUpstreamSuccessor>, WorthQueryOutputDemandDenial>
    {
        let prerequisites = self.declared_required_predecessors(selected.key())?;
        for key in prerequisites.keys {
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(head) =
                super::super::super::required_stop::lineage_head(&state.records, &key)?
            else {
                continue;
            };
            drop(state);
            if let Some(held) = self.take_published_held_successor(&head)? {
                return Ok(Some(
                    super::super::super::RequiredUpstreamSuccessor::issued(held),
                ));
            }
        }
        Ok(None)
    }
}
