//! Original/postcondition association issued by the exact retained record and mark image.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl;
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use worth_execution::ChargedBytes;
use worth_execution::ExecutionArray;
use worth_execution::ExecutionArrayBuilder;
use worth_relational::facade::history::RelationalDescriptiveTouchGraph;

/// Candidate-derived coverage and its allocation custody move together.
pub(in crate::domain_computation::primary_graph) struct SelectedPostconditionRebase {
    pub(in crate::domain_computation::primary_graph) ordinals: OrdSet<usize>,
    pub(in crate::domain_computation::primary_graph) basis:
        super::super::settlement::PostingRebaseBasis,
}

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
    fn posting_basis(
        &self,
        selected: &OrdSet<usize>,
    ) -> super::super::settlement::PostingRebaseBasis {
        super::super::settlement::PostingRebaseBasis::from_carry(
            Arc::clone(&self.changes.identity),
            self.changes.postconditions.clone(),
            self.changes.storage.clone(),
            selected.clone(),
        )
    }

    pub(in crate::domain_computation::primary_graph) fn matches_publication_source(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<bool, CompanionPreflightStop> {
        self.changes.matches_publication_source(runtime, snapshot)
    }
    pub(in crate::domain_computation::primary_graph) fn unchanged_payload_bytes(
        &self,
        selected: &OrdSet<usize>,
    ) -> Result<u64, CompanionPreflightStop> {
        let mut bytes = self.changes.payload_bytes;
        for ordinal in selected
            .iter()
            .copied()
            .filter(|ordinal| *ordinal < self.prefix)
            .chain(self.prefix..self.changes.postconditions.len())
        {
            let removed = self.changes.postconditions[ordinal].additional_charged_bytes();
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
    ) -> Result<Option<SelectedPostconditionRebase>, StoreDenial> {
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
                    .checked_add(
                        self.changes
                            .storage
                            .projection
                            .ordinals()
                            .get(&key)
                            .map_or(0, OrdSet::len),
                    )
                    .ok_or(StoreDenial::Representability)?;
                Ok(())
            })?;
        }
        control.check_live()?;
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
                if let Some(ordinals) = self.changes.storage.projection.ordinals().get(&key) {
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
        let basis = self.posting_basis(&selected);
        Ok(Some(SelectedPostconditionRebase {
            ordinals: selected,
            basis,
        }))
    }
}
fn visit(
    touch: &worth_relational::facade::history::RelationalDescriptiveTouch,
    control: StorageControl<'_, '_>,
    mut emit: impl FnMut(super::super::fact_key::FactPostingKey) -> Result<(), StoreDenial>,
) -> Result<(), StoreDenial> {
    control.check_live()?;
    super::super::touch_keys::visit(touch, |key| {
        crate::domain_computation::primary_graph::fact_verification_observation::rebase_touch_key();
        control.check_live()?;
        emit(key)
    })
}
