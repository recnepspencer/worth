//! One paid borrowed index replaces repeated scans of the same candidate.
use super::InvariantExecutionContext;
use crate::identity::data::{KindId, PartitionId};
use crate::symbols::data::ClientKey;
use crate::transactions::data::{CreateIntent, CreatedEntityRef, MutationIntent};

type CreatedKey<'a> = (PartitionId, KindId, &'a ClientKey);
pub(super) type CreatedKinds<'a> = Vec<CreatedKey<'a>>;

impl<'runtime, 'budget> InvariantExecutionContext<'runtime, 'budget> {
    pub(crate) fn created_entity_kind(&self, created: &CreatedEntityRef) -> Option<KindId> {
        let rows = self
            .created_kinds
            .get_or_init(|| self.index_created_kinds())
            .as_ref()?;
        let wanted = (created.partition_id, created.kind_id, &created.client_key);
        let mut live = true;
        let index = rows.binary_search_by(|row| {
            if !live || !self.checkpoint(1) {
                live = false;
                return std::cmp::Ordering::Greater;
            }
            row.cmp(&wanted)
        });
        if !live {
            return None;
        }
        index.ok().map(|index| rows[index].1)
    }

    fn index_created_kinds(&self) -> Option<CreatedKinds<'runtime>> {
        let plan = self.merged_plan()?;
        let mut count = 0usize;
        for intent in &plan.merged_intents {
            if !self.checkpoint(1) {
                return None;
            }
            count = count.checked_add(match intent {
                MutationIntent::Create(CreateIntent::Entity(_)) => 1,
                MutationIntent::Create(CreateIntent::BulkEntities(spec)) => spec.client_keys.len(),
                _ => 0,
            })?;
        }
        let bytes = (count as u64).checked_mul(std::mem::size_of::<CreatedKey<'_>>() as u64)?;
        if !self.claim_scratch(bytes) {
            return None;
        }
        let mut rows = Vec::with_capacity(count);
        for intent in &plan.merged_intents {
            match intent {
                MutationIntent::Create(CreateIntent::Entity(spec)) => {
                    if !self.checkpoint(1) {
                        return None;
                    }
                    rows.push((spec.partition_id, spec.kind_id, &spec.client_key));
                }
                MutationIntent::Create(CreateIntent::BulkEntities(spec)) => {
                    for key in &spec.client_keys {
                        if !self.checkpoint(1) {
                            return None;
                        }
                        rows.push((spec.partition_id, spec.kind_id, key));
                    }
                }
                _ => {}
            }
        }
        // The sort allocates no heap. Admit its comparison bound before it runs.
        let comparisons =
            (count as u64).checked_mul(u64::from(usize::BITS - count.max(1).leading_zeros()))?;
        if !self.checkpoint(comparisons) {
            return None;
        }
        rows.sort_unstable();
        Some(rows)
    }
}
