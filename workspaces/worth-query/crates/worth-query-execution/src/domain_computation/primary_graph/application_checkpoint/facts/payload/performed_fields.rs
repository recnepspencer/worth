//! Pre-effect immutable field evidence, excluding Native publication/read custody.
use super::{Fact, FactPayload, FactSequence, StorageControl};
use crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial;

impl FactPayload {
    /// O(facts + copied field bodies + balanced join paths). Copies only the
    /// genuine completed handler reads and original field revisions, funded by
    /// that producer request before any effect. No view retains a parent body.
    pub(in crate::domain_computation::primary_graph) fn copy_performed_field_reads(
        facts: &Self,
        control: StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        let mut reads = facts.iter();
        let mut result: Option<Self> = None;
        loop {
            let mut chunk = FieldReadChunk {
                fields: [None; 32],
                count: 0,
            };
            while chunk.count < chunk.fields.len() {
                let Some(fact) = reads.next() else { break };
                control.check_live()?;
                if matches!(
                    fact,
                    Fact::Field { .. }
                        | Fact::AbsentField { .. }
                        | Fact::OptionalAspectAbsent { .. }
                        | Fact::SourceFieldRevision { .. }
                ) {
                    chunk.fields[chunk.count] = Some(fact);
                    chunk.count += 1;
                }
            }
            if chunk.count == 0 && result.is_some() {
                break;
            }
            let copied = Self::copy_funded(&chunk, control)?;
            result = Some(match result {
                None => copied,
                Some(previous) => previous.append_completed(copied, control)?,
            });
            if chunk.count < chunk.fields.len() {
                break;
            }
        }
        result.ok_or(StoreDenial::ConflictingBody)
    }
}

/// Bounded borrowed authoring scratch, never retained in the published evidence.
struct FieldReadChunk<'a> {
    fields: [Option<&'a Fact>; 32],
    count: usize,
}
impl FactSequence for FieldReadChunk<'_> {
    fn len(&self) -> usize {
        self.count
    }
    fn get(&self, ordinal: usize) -> Option<&Fact> {
        self.fields[..self.count].get(ordinal).copied().flatten()
    }
    fn iter(&self) -> impl DoubleEndedIterator<Item = &Fact> + ExactSizeIterator + Clone {
        self.fields[..self.count]
            .iter()
            .map(|fact| fact.expect("filled field scratch"))
    }
}

#[cfg(test)]
mod tests;
