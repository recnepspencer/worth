use std::collections::BTreeSet;

use worth_relational::facade::identity::EntityId;

/// Exact entity identities reached while one pinned invariant projection ran.
///
/// This is carried proof, not a caller-authored touch assertion. Only the
/// installation-owned projection reader can add identities.
#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryRealizedProjectionScope {
    entity_ids: BTreeSet<EntityId>,
    /// Every identity one owner call of a partitioned computation reached,
    /// already reached or not, while that call holds the reader.
    call: Option<BTreeSet<EntityId>>,
    carried: Vec<(
        std::sync::Arc<super::ComputationCallSequence>,
        std::ops::Range<usize>,
    )>,
    carried_memory: Option<
        crate::domain_computation::primary_graph::application_contribution::QueryMemoryReservation,
    >,
}

impl WorthQueryRealizedProjectionScope {
    pub(super) fn record(&mut self, entity_id: EntityId) {
        self.entity_ids.insert(entity_id);
        if let Some(call) = &mut self.call {
            call.insert(entity_id);
        }
    }

    pub(super) fn record_relation(&mut self, from: EntityId, to: EntityId) {
        self.record(from);
        self.record(to);
    }

    /// Begins collecting what one owner call reaches.
    pub(super) fn begin_call(&mut self) {
        self.call = Some(BTreeSet::new());
    }

    /// Everything the call begun last reached, in order.
    pub(super) fn end_call(&mut self) -> Vec<EntityId> {
        self.call.take().unwrap_or_default().into_iter().collect()
    }

    /// Admits identities an earlier run of the same call reached.
    pub(super) fn extend(&mut self, entities: &[EntityId]) {
        self.entity_ids.extend(entities.iter().copied());
    }

    pub(super) fn carry_range(&mut self, sequence:&std::sync::Arc<super::ComputationCallSequence>, range:std::ops::Range<usize>, execution:&crate::domain_computation::primary_graph::application_contribution::QueryRequestExecution<'_>)->Result<(),crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial>{
        use crate::domain_computation::primary_graph::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
        let bytes = self
            .carried
            .len()
            .checked_add(1)
            .and_then(|n| {
                n.checked_mul(
                    4 * std::mem::size_of::<(
                        std::sync::Arc<super::ComputationCallSequence>,
                        std::ops::Range<usize>,
                    )>(),
                )
            })
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Resource::CapacityOverflow)?;
        match &mut self.carried_memory {
            Some(memory) => memory.resize(bytes)?,
            None => self.carried_memory = Some(execution.reserve(bytes)?),
        }
        self.carried.push((std::sync::Arc::clone(sequence), range));
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn contains(
        &self,
        entity_id: EntityId,
    ) -> bool {
        self.entity_ids.contains(&entity_id)
            || self
                .carried
                .iter()
                .any(|(sequence, range)| sequence.contains_in(entity_id, range))
    }
}
