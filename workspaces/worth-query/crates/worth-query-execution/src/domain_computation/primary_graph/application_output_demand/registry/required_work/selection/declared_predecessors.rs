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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        for key in &self.keys {
            admission
                .charge_external_work(10)
                .map_err(|_| work_denial())?;
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Vec<SelectedReadyReadmission>, WorthQueryOutputDemandDenial> {
        let declared = self.declared_required_predecessors(selected.key(), admission)?;
        let count = declared.keys.len();
        let bytes = count
            .checked_mul(std::mem::size_of::<SelectedReadyReadmission>())
            .ok_or_else(capacity_denial)?;
        admission
            .admit_read_scratch(u64::try_from(bytes).map_err(|_| capacity_denial())?)
            .map_err(|_| capacity_denial())?;
        admission
            .charge_external_work(u64::try_from(bytes).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
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
            state.charge_record_lookup(&key, admission)?;
            let Some(head) =
                super::super::super::required_stop::lineage_head(&state.records, &key, admission)?
            else {
                continue;
            };
            state.charge_record_lookup(&head, admission)?;
            charge_required_key_lookup(&state, &head, admission)?;
            if let Some(member) =
                super::super::pending_readmission::ready_member(&state, &head, admission)?
            {
                ready.push(member);
            }
        }
        Ok(ready)
    }
    pub(in crate::domain_computation::primary_graph) fn declared_required_predecessors(
        &self,
        selected: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<DeclaredRequiredPredecessors, WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(selected, admission)?;
        let Some(record) = state.records.get(selected) else {
            return Ok(DeclaredRequiredPredecessors { keys: Vec::new() });
        };
        let count = record.prerequisites.len();
        let bytes = count
            .checked_mul(std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>())
            .ok_or_else(capacity_denial)?;
        admission
            .admit_read_scratch(u64::try_from(bytes).map_err(|_| capacity_denial())?)
            .map_err(|_| capacity_denial())?;
        admission
            .charge_external_work(
                u64::try_from(bytes.checked_add(count).ok_or_else(work_denial)?)
                    .map_err(|_| work_denial())?,
            )
            .map_err(|_| work_denial())?;
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<super::super::super::RequiredUpstreamSuccessor>, WorthQueryOutputDemandDenial>
    {
        let prerequisites = self.declared_required_predecessors(selected.key(), admission)?;
        for key in prerequisites.keys {
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(head) =
                super::super::super::required_stop::lineage_head(&state.records, &key, admission)?
            else {
                continue;
            };
            state.charge_record_lookup(&head, admission)?;
            let published = state.records.get(&head).is_some_and(|row|
                matches!(&row.state, super::super::super::DemandState::Output(output)
                    if matches!(output.checkpoint, Some(super::super::super::WorthQueryOutputCheckpoint::Published { .. }))));
            drop(state);
            if published {
                if let Some(held) = self.take_held_successor(&head, admission)? {
                    return Ok(Some(
                        super::super::super::RequiredUpstreamSuccessor::issued(held),
                    ));
                }
            }
        }
        Ok(None)
    }
}
