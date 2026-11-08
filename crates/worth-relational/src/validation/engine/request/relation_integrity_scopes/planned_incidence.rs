//! Index the proposal's changed edges once, never once per visited vertex.
use super::PlannedRelationEdge;
use crate::transactions::data::{EntityReference, TransactionCommitError};
use std::collections::BTreeMap;

type Prepare<'a> = dyn Fn(u64, u64) -> Result<(), TransactionCommitError> + 'a;
type Incidence = BTreeMap<EntityReference, Vec<EntityReference>>;
pub(super) struct PlannedIncidence {
    incoming: Incidence,
    outgoing: Incidence,
    lookup_work: u64,
}
impl PlannedIncidence {
    pub(super) fn new(
        edges: &[PlannedRelationEdge],
        prepare: &Prepare<'_>,
    ) -> Result<Self, TransactionCommitError> {
        let lookup_work = 1 + u64::from((edges.len() + 1).ilog2());
        let mut index = Self {
            incoming: BTreeMap::new(),
            outgoing: BTreeMap::new(),
            lookup_work,
        };
        for edge in edges {
            let bytes = super::successor_scope::reference_quote(&edge.source)
                .saturating_add(super::successor_scope::reference_quote(&edge.target));
            // Both directional keys and vectors coexist; Vec growth can double
            // backing. Admit their real reference/name layouts before cloning.
            prepare(lookup_work.saturating_mul(2), bytes.saturating_mul(4))?;
            index
                .incoming
                .entry(edge.target.clone())
                .or_default()
                .push(edge.source.clone());
            index
                .outgoing
                .entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
        }
        Ok(index)
    }
    pub(super) fn predecessors(
        &self,
        entity: &EntityReference,
        prepare: &Prepare<'_>,
    ) -> Result<&[EntityReference], TransactionCommitError> {
        prepare(self.lookup_work, 0)?;
        Ok(self.incoming.get(entity).map_or(&[], Vec::as_slice))
    }
    pub(super) fn successors(
        &self,
        entity: &EntityReference,
        prepare: &Prepare<'_>,
    ) -> Result<&[EntityReference], TransactionCommitError> {
        prepare(self.lookup_work, 0)?;
        Ok(self.outgoing.get(entity).map_or(&[], Vec::as_slice))
    }
}
