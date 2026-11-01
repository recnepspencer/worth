//! Cold locator backing keeps one shared admission after descriptor cloning.
use super::StrategyMergeDescriptor;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[cfg(test)]
mod tests;

impl StrategyMergeDescriptor {
    pub(crate) fn intent_scope_targets_control_bytes() -> Option<u64> {
        let layout = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<ExecutionMemoryReservation>())
            .ok()?
            .0
            .pad_to_align();
        u64::try_from(layout.size()).ok()
    }

    pub(crate) fn retain_intent_scope_targets_memory(
        &mut self,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Result<(), Option<ExecutionMemoryReservation>> {
        let Some(memory) = memory else {
            return Ok(());
        };
        if self.intent_scope_targets_memory.is_some()
            || Arc::get_mut(&mut self.intent_scope_targets).is_none()
        {
            return Err(Some(memory));
        }
        self.intent_scope_targets_memory = Some(Arc::new(memory));
        Ok(())
    }
}

impl PartialEq for StrategyMergeDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.strategy_id == other.strategy_id
            && self.descriptor_digest == other.descriptor_digest
            && self.semantic_name == other.semantic_name
            && self.family_name == other.family_name
            && self.version == other.version
            && self.intent_name == other.intent_name
            && self.intent_scope_digest == other.intent_scope_digest
            && self.intent_scope_targets == other.intent_scope_targets
            && self.merge_semantics == other.merge_semantics
            && self.lowering_summary_digest == other.lowering_summary_digest
    }
}
impl Eq for StrategyMergeDescriptor {}

impl std::fmt::Debug for StrategyMergeDescriptor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("StrategyMergeDescriptor")
            .field("strategy_id", &self.strategy_id)
            .field("descriptor_digest", &self.descriptor_digest)
            .field("semantic_name", &self.semantic_name)
            .field("family_name", &self.family_name)
            .field("version", &self.version)
            .field("intent_name", &self.intent_name)
            .field("intent_scope_digest", &self.intent_scope_digest)
            .field("intent_scope_targets", &self.intent_scope_targets)
            .field("merge_semantics", &self.merge_semantics)
            .field("lowering_summary_digest", &self.lowering_summary_digest)
            .finish()
    }
}
