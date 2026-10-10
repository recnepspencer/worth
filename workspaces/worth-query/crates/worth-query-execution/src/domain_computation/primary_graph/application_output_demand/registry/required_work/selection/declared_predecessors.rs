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
        selected: &SelectedReadyReadmission,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        for key in &self.keys {
            admission
                .charge_external_work(10)
                .map_err(|_| work_denial())?;
            if key.same_occurrence(selected.key()) {
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
        let declared = self.declared_required_predecessors(selected, admission)?;
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
        selected: &SelectedReadyReadmission,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<DeclaredRequiredPredecessors, WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(selected.key(), admission)?;
        let Some(record) = state.records.get(selected.key()) else {
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
