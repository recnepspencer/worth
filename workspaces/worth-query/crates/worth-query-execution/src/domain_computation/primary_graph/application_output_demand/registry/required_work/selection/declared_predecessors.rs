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
