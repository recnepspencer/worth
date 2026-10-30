//! Native installs associations only from its selected reservation and judged set.
use super::{
    CompanionPreflightStop as Stop, PublicationCompanionPreflight, ReservedCompanionBranchCell,
};
use crate::validation::candidate_source::{
    ManagedIndexBranchImage, ManagedIndexPublishedSourceRoot,
};
use std::sync::Arc;

impl PublicationCompanionPreflight<'_> {
    pub fn prepare_managed_source_image(
        &self,
        selected: &ReservedCompanionBranchCell<ManagedIndexBranchImage>,
    ) -> Result<Arc<ManagedIndexBranchImage>, Stop> {
        if selected.core.runtime_instance_id != self.binding.runtime_instance_id
            || selected.core.branch_id != self.binding.branch_id
            || selected.core.registration_generation != self.binding.registration_generation
            || selected.current().root_id() != self.binding.expected_root_id
        {
            return Err(Stop::SelectedSourceMismatch);
        }
        let mut next = (**selected.current().payload()).clone();
        for root in self.managed_sources().roots() {
            crate::authority::commit::pipeline::consult_stage_control(
                self.execution,
                crate::authority::commit::pipeline::CommitStage::Publication,
            )
            .map_err(|cause| Stop::Execution(cause.into()))?;
            self.checkpoint()?;
            let key = (root.program().clone(), root.scope());
            let Some(payload) = root.proposed() else {
                next.roots.remove(&key);
                continue;
            };
            // Reuse requires this actual Native preparation's unchanged Source
            // disposition, not equal row values or a caller's old root number.
            if !root.source_changed()
                && next.roots.get(&key).is_some_and(|previous| {
                    previous.availability == root.physical_availability()
                        && Arc::ptr_eq(&previous.payload, payload)
                })
            {
                continue;
            }
            let bytes = std::alloc::Layout::new::<[usize; 2]>()
                .extend(std::alloc::Layout::new::<ManagedIndexPublishedSourceRoot>())
                .ok()
                .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
                .ok_or(Stop::PreparationMemoryCounterOverflow)?;
            let memory = self
                .execution
                .reserve_retained_memory(bytes)
                .map_err(|cause| Stop::Execution(cause.into()))?;
            next.roots.insert(
                key,
                Arc::new(ManagedIndexPublishedSourceRoot {
                    program: root.program().clone(),
                    scope: root.scope(),
                    payload: Arc::clone(payload),
                    availability: root.physical_availability(),
                    revision: self.binding.next_commit_id,
                    _memory: memory,
                }),
            );
        }
        self.checkpoint()?;
        Ok(Arc::new(next))
    }
}
