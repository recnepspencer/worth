use super::{CausalLink, NodeExplanation, RewiringDependency, RewiringSummary, ScopeProvenance};
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial as Denial,
};
impl RetainedStorageMeasurement for NodeExplanation {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            node: _,
            materialization_mode: _,
            state: _,
            dirty_aspects: _,
            contract_reads: _,
            contract_produces: _,
            contract_partition_scope,
            required_context,
            condition,
            historical_artifact_record,
            execution_record_id: _,
            semantic_segment_id: _,
            output_identity,
            output_change: _,
            changed_regions,
            propagation_suppressed: _,
            memoized_origin: _,
            reuse_basis,
            reuse_origin: _,
            reuse_certification,
            upstream,
            causal_links,
            rewiring,
            causality,
        } = self;
        contract_partition_scope
            .retained_heap_charge(work)?
            .checked_add(required_context.retained_heap_charge(work)?)?
            .checked_add(condition.retained_heap_charge(work)?)?
            .checked_add(historical_artifact_record.retained_heap_charge(work)?)?
            .checked_add(output_identity.retained_heap_charge(work)?)?
            .checked_add(changed_regions.retained_heap_charge(work)?)?
            .checked_add(reuse_basis.retained_heap_charge(work)?)?
            .checked_add(reuse_certification.retained_heap_charge(work)?)?
            .checked_add(upstream.retained_heap_charge(work)?)?
            .checked_add(causal_links.retained_heap_charge(work)?)?
            .checked_add(rewiring.retained_heap_charge(work)?)?
            .checked_add(causality.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for ScopeProvenance {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            source_scope,
            validation_scope,
            kind: _,
            note,
        } = self;
        source_scope
            .retained_heap_charge(work)?
            .checked_add(validation_scope.retained_heap_charge(work)?)?
            .checked_add(note.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for CausalLink {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            source: _,
            aspect: _,
            disposition: _,
            kind,
            scope,
            cached_version: _,
            current_version: _,
            comparator,
            reason,
            note,
        } = self;
        kind.retained_heap_charge(work)?
            .checked_add(scope.retained_heap_charge(work)?)?
            .checked_add(comparator.retained_heap_charge(work)?)?
            .checked_add(reason.retained_heap_charge(work)?)?
            .checked_add(note.retained_heap_charge(work)?)
    }
}
impl RetainedStorageMeasurement for RewiringDependency {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self {
            source: _,
            aspect: _,
            subscription,
        } = self;
        subscription.retained_heap_charge(work)
    }
}
impl RetainedStorageMeasurement for RewiringSummary {
    fn retained_heap_charge(&self, work: &mut Work) -> Result<Charge, Denial> {
        work.visit()?;
        let Self { added, removed } = self;
        added
            .retained_heap_charge(work)?
            .checked_add(removed.retained_heap_charge(work)?)
    }
}

mod causes;

#[cfg(test)]
mod tests;
