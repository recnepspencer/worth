use super::{ExplanationFact, ProvenanceEdge, ProvenanceFact, ProvenanceVertex};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for ExplanationFact {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            node: _,
            explanation,
            compact_projection: _,
            materialization_mode: _,
            execution_record_id: _,
            semantic_segment_id: _,
            state,
            upstream_count: _,
            propagation_suppressed: _,
            changed_region_count: _,
            output_change,
        } = self;
        explanation
            .retained_heap_charge(work)?
            .checked_add(state.retained_heap_charge(work)?)?
            .checked_add(output_change.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for ProvenanceFact {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            node: _,
            materialization_mode: _,
            execution_record_id: _,
            semantic_segment_id: _,
            vertices,
            edges,
            causal_links,
            rewiring,
            propagation_suppressed: _,
            causality_kind,
        } = self;
        vertices
            .retained_heap_charge(work)?
            .checked_add(edges.retained_heap_charge(work)?)?
            .checked_add(causal_links.retained_heap_charge(work)?)?
            .checked_add(rewiring.retained_heap_charge(work)?)?
            .checked_add(causality_kind.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for ProvenanceVertex {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            node: _,
            role: _,
            state,
        } = self;
        state.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for ProvenanceEdge {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            kind,
            source: _,
            aspect: _,
            subscription,
            cached_version: _,
            current_version: _,
            comparator,
            reason,
        } = self;
        kind.retained_heap_charge(work)?
            .checked_add(subscription.retained_heap_charge(work)?)?
            .checked_add(comparator.retained_heap_charge(work)?)?
            .checked_add(reason.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for super::ProvenanceEdgeKind {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        match self {
            Self::Changed
            | Self::SkippedByComparator
            | Self::ConditionDeferred
            | Self::Clean
            | Self::MissingSnapshot
            | Self::DependencyRemoved => Ok(Charge::ZERO),
        }
    }
}

#[cfg(test)]
mod tests;
