use super::super::{
    denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use worth_foundational::facade::AspectFieldLocator;
use worth_query_installation::facade::WorthQueryWorkflowHistoryReconstructionBudget;
use worth_relational::facade::{
    identity::{EntityId, KindId},
    runtime::{ProjectionAspectScope, RelationalRuntime},
    snapshots::SnapshotHandle,
};

// Logical reservation, not RSS: one reusable observation scratch buffer plus
// per-transition inventory/projection vectors, progress tree nodes and replay
// wrappers. Variable native strings and replay paths are charged separately,
// before cloning, at their maximum simultaneous owned-copy multiplicity.
const SCRATCH_CHARGE: usize = 16 * 1024;
const TRANSITION_CHARGE: usize = 4096;
const OWNED_COPY_MULTIPLICITY: usize = 4;

pub(super) struct HistoryReconstructionCharge {
    budget: WorthQueryWorkflowHistoryReconstructionBudget,
    bytes: usize,
}

impl HistoryReconstructionCharge {
    pub(super) fn new(
        budget: WorthQueryWorkflowHistoryReconstructionBudget,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial> {
        let mut charge = Self { budget, bytes: 0 };
        charge.reserve(SCRATCH_CHARGE)?;
        Ok(charge)
    }

    pub(super) fn inventory_limit(&self, retained_limit: usize) -> usize {
        retained_limit
            .min(self.budget.maximum_transition_visits() as usize)
            .min((self.maximum_bytes() - self.bytes) / TRANSITION_CHARGE)
    }

    pub(super) fn reserve_inventory(
        &mut self,
        count: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.reserve(count.checked_mul(TRANSITION_CHARGE).ok_or_else(exhausted)?)
    }

    pub(super) fn reserve_path(
        &mut self,
        path: &str,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.reserve(
            path.len()
                .checked_mul(OWNED_COPY_MULTIPLICITY)
                .ok_or_else(exhausted)?,
        )
    }

    pub(super) fn reserve_record(
        &mut self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        entity: EntityId,
        kind: KindId,
        locator: &AspectFieldLocator,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let aspect = locator.aspect().aspect_key();
        runtime
            .read_truth()
            .project_snapshot(snapshot)
            .ok_or_else(|| denial("history reconstruction snapshot is unavailable"))?
            .entity_record_with_projection_scope(
                entity,
                ProjectionAspectScope::whole_aspects([aspect.clone()]),
                |record| {
                    if record.kind_id() != kind {
                        return None;
                    }
                    let value = record.struct_aspect_value(aspect)?;
                    Some(value.fields().try_for_each(|(_, value)| {
                        let bytes = value
                            .owned_allocation_capacity_bytes()
                            .checked_mul(OWNED_COPY_MULTIPLICITY)
                            .ok_or_else(exhausted)?;
                        self.reserve(bytes)
                    }))
                },
            )
            .ok_or_else(|| denial("history reconstruction record is unavailable"))?
    }

    pub(super) const fn bytes(&self) -> usize {
        self.bytes
    }

    fn maximum_bytes(&self) -> usize {
        usize::try_from(self.budget.maximum_charge_bytes()).unwrap_or(usize::MAX)
    }

    fn reserve(&mut self, bytes: usize) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let next = self.bytes.checked_add(bytes).ok_or_else(exhausted)?;
        if next > self.maximum_bytes() {
            return Err(exhausted());
        }
        self.bytes = next;
        Ok(())
    }
}

pub(super) fn exhausted() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowHistoryReconstructionBudgetExceeded,
        "workflow history reconstruction memory ceiling",
    )
}

pub(super) fn inventory_exhausted() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowHistoryReconstructionBudgetExceeded,
        "workflow history reconstruction inventory ceiling",
    )
}
