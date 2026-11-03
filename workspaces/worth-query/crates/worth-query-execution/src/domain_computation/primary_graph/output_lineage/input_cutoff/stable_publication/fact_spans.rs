//! Exact alias-local ordinal layout; descriptive storage, never currentness.
use crate::domain_computation::primary_graph::{
    application_attempt::retained_decision_facts::{StorageControl, StoreDenial},
    application_checkpoint::facts::FactPayload,
};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph::output_lineage) struct StableFactSpans {
    body: FactPayload,
    handler_end: usize,
    witness_end: usize,
}
impl StableFactSpans {
    pub(in crate::domain_computation::primary_graph::output_lineage) fn new(
        body: FactPayload,
        handler_count: usize,
        witness_count: usize,
    ) -> Option<Self> {
        let witness_end = handler_count.checked_add(witness_count)?;
        (witness_end <= body.len()).then_some(Self {
            body,
            handler_end: handler_count,
            witness_end,
        })
    }
    pub(in crate::domain_computation::primary_graph::output_lineage) fn matches(
        &self,
        body: &FactPayload,
    ) -> bool {
        self.body.same_backing(body)
    }
    pub(in crate::domain_computation::primary_graph::output_lineage) fn performed_sequence(
        &self,
        body: &FactPayload,
        handler_count: usize,
        control: StorageControl<'_, '_>,
    ) -> Result<FactPayload, StoreDenial> {
        control.check_live()?;
        if !self.matches(body) || handler_count != self.handler_end {
            return Err(StoreDenial::ConflictingBody);
        }
        let prefix = body
            .checked_slice(0..self.handler_end)
            .ok_or(StoreDenial::ConflictingBody)?;
        let suffix = body
            .checked_slice(self.witness_end..body.len())
            .ok_or(StoreDenial::ConflictingBody)?;
        prefix.append_completed(suffix, control)
    }
}

#[cfg(test)]
mod tests;
