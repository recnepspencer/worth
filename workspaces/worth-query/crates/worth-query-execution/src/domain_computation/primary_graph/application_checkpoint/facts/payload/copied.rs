//! New deep copies keep their own leaf-local grants, never a parent body.
use super::{chunk::deep_clone_bytes, FactPayload, FactSequence, StorageControl};
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;
use worth_execution::ExecutionArrayBuilder;

impl FactPayload {
    /// O(new facts * source depth + output join paths). Each bounded leaf
    /// is admitted before its first copy.
    /// This funds storage only; it grants no original-read or source authority.
    pub(in crate::domain_computation::primary_graph) fn copy_funded(
        facts: &(impl FactSequence + ?Sized),
        control: StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        control.check_live()?;
        let mut result: Option<Self> = None;
        for start in (0..facts.len().max(1)).step_by(32) {
            let end = start.saturating_add(32).min(facts.len());
            let quote = (start..end)
                .try_fold(0_u64, |sum, ordinal| {
                    sum.checked_add(deep_clone_bytes(facts.get(ordinal)?)?)
                })
                .ok_or(StoreDenial::Representability)?;
            let heaps = control.policy().reserve_memory(quote);
            control.check_live()?;
            let heaps = heaps?;
            // Inline output drops before its deep-heap grant on every refusal.
            let mut output = ExecutionArrayBuilder::allocate(end - start, control.policy())?;
            for ordinal in start..end {
                control.check_live()?;
                output.push(
                    facts
                        .get(ordinal)
                        .ok_or(StoreDenial::ConflictingBody)?
                        .clone(),
                )?;
                crate::domain_computation::primary_graph::fact_verification_observation::native_payload_fact_move();
            }
            control.check_live()?;
            let leaf = Self::copied_native(output.seal()?, heaps, None);
            result = Some(match result {
                None => leaf,
                Some(root) => root.append_completed(leaf, control)?,
            });
        }
        control.check_live()?;
        result.ok_or(StoreDenial::ConflictingBody)
    }
}

#[cfg(test)]
mod tests;
