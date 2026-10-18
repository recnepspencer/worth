//! Original/postcondition association issued by the exact retained record and mark image.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::{StorageControl, StoreDenial},
    application_checkpoint::facts::FactPayload,
};
use worth_execution::{ExecutionArray, ExecutionArrayBuilder, ExecutionMemoryReservation};
use worth_relational::facade::history::RelationalDescriptiveTouchGraph;

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct PostconditionCarry {
    changes: ComputationChanges,
    prefix: usize,
    fresh: Arc<ExecutionArray<usize>>,
}
impl std::fmt::Debug for PostconditionCarry {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("PostconditionCarry")
            .field("prefix", &self.prefix)
            .finish_non_exhaustive()
    }
}
impl ComputationChanges {
    pub(in crate::domain_computation::primary_graph) fn carry(
        &self,
        original: &FactPayload,
        prefix: usize,
        fresh: &OrdSet<usize>,
        snapshot: &SnapshotHandle,
        control: StorageControl<'_, '_>,
    ) -> Result<Option<PostconditionCarry>, StoreDenial> {
        control.check_live()?;
        if self.snapshot != *snapshot {
            crate::domain_computation::primary_graph::fact_verification_observation::native_completion_decline("postcondition-snapshot");
            return Ok(None);
        }
        if self
            .original
            .as_ref()
            .is_none_or(|held| !held.same_backing(original))
        {
            crate::domain_computation::primary_graph::fact_verification_observation::native_completion_decline("postcondition-original-backing");
            return Ok(None);
        }
        if original.len() != self.postconditions.len() || prefix > original.len() {
            crate::domain_computation::primary_graph::fact_verification_observation::native_completion_decline("postcondition-shape");
            return Ok(None);
        }
        let mut granted = ExecutionArrayBuilder::allocate(fresh.len(), control.policy())?;
        for &ordinal in fresh {
            control.check_live()?;
            granted.push(ordinal)?;
        }
        Ok(Some(PostconditionCarry {
            changes: self.clone(),
            prefix,
            fresh: Arc::new(granted.seal()?),
        }))
    }
}
impl PostconditionCarry {
    pub(in crate::domain_computation::primary_graph) fn matches_publication_source(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &SnapshotHandle,
        admission: &mut super::super::InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        self.changes
            .matches_publication_source(runtime, snapshot, admission)
    }
    pub(in crate::domain_computation::primary_graph) fn unchanged_payload_bytes(
        &self,
        selected: &OrdSet<usize>,
        admission: &mut super::super::InvalidationEditAdmission,
    ) -> Result<u64, CompanionPreflightStop> {
        let mut bytes = self.changes.payload_bytes;
        for ordinal in selected
            .iter()
            .copied()
            .filter(|ordinal| *ordinal < self.prefix)
            .chain(self.prefix..self.changes.postconditions.len())
        {
            admission.charge_external_work(1)?;
            let removed = super::super::fact_retention::fact_payload_bytes(
                &self.changes.postconditions[ordinal],
                admission,
            )?;
            bytes = bytes
                .checked_sub(removed)
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        }
        Ok(bytes)
    }
    pub(in crate::domain_computation::primary_graph) fn prefix(&self) -> usize {
        self.prefix
    }
    pub(in crate::domain_computation::primary_graph) fn baseline(&self) -> &FactPayload {
        &self.changes.postconditions
    }
    /// Both passes visit only candidate touches and their matching local postings.
    /// The original image owns the projection; no independent reverse index exists.
    pub(in crate::domain_computation::primary_graph) fn select(
        &self,
        touches: &RelationalDescriptiveTouchGraph,
        count: usize,
        moved: &[usize],
        control: StorageControl<'_, '_>,
    ) -> Result<Option<(OrdSet<usize>, Option<ExecutionMemoryReservation>)>, StoreDenial> {
        let Some(touches) = touches.exact_touches() else {
            return Ok(None);
        };
        if count < self.prefix {
            return Ok(None);
        }
        let mut maximum = self
            .changes
            .ordinals
            .len()
            .checked_add(self.fresh.len())
            .and_then(|n| n.checked_add(count - self.prefix))
            .and_then(|n| n.checked_add(moved.len()))
            .ok_or(StoreDenial::Representability)?;
        for touch in touches {
            control.check_live()?;
            visit(touch, control, |key| {
                maximum = maximum
                    .checked_add(self.changes.postings.get(&key).map_or(0, OrdSet::len))
                    .ok_or(StoreDenial::Representability)?;
                Ok(())
            })?;
        }
        let bytes = super::super::index_capacity::ordered_edit_bytes::<usize, ()>(maximum)
            .and_then(|per| per.checked_mul(u64::try_from(maximum).ok()?))
            .ok_or(StoreDenial::Representability)?;
        let memory = control.policy().reserve_memory(bytes);
        control.check_live()?;
        let memory = memory?;
        let mut selected = OrdSet::new();
        for ordinal in self
            .changes
            .ordinals
            .iter()
            .chain(self.fresh.iter())
            .chain(moved.iter())
        {
            control.check_live()?;
            if *ordinal < self.prefix {
                selected.insert(*ordinal);
            }
        }
        for ordinal in self.prefix..count {
            control.check_live()?;
            selected.insert(ordinal);
        }
        for touch in touches {
            visit(touch, control, |key| {
                if let Some(ordinals) = self.changes.postings.get(&key) {
                    for &ordinal in ordinals {
                        control.check_live()?;
                        if ordinal < self.prefix {
                            selected.insert(ordinal);
                        }
                    }
                }
                Ok(())
            })?;
        }
        Ok(Some((selected, memory)))
    }
}
fn visit(
    touch: &worth_relational::facade::history::RelationalDescriptiveTouch,
    control: StorageControl<'_, '_>,
    mut emit: impl FnMut(super::super::fact_key::FactPostingKey) -> Result<(), StoreDenial>,
) -> Result<(), StoreDenial> {
    let held = std::cell::RefCell::new(None);
    super::super::touch_keys::visit(
        touch,
        |bytes| {
            control.check_live()?;
            let bytes = u64::try_from(bytes).map_err(|_| StoreDenial::Representability)?;
            let grant = control.policy().reserve_memory(bytes);
            control.check_live()?;
            *held.borrow_mut() = grant?;
            Ok(())
        },
        |key| {
            crate::domain_computation::primary_graph::fact_verification_observation::rebase_touch_key();
            control.check_live()?;
            let result = emit(key);
            held.borrow_mut().take();
            result
        },
    )
}
