use super::{
    PreparedDependencyCapture, PreparedEvaluation, PreparedKeyedContext, PreparedTraceData,
};
use crate::data::output::NodeEvaluationResult;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageMeasurement,
    RetainedStoragePreparation as Preparation, RetainedStoragePreparationDenial as Denial,
};

impl worth_execution::ChargedBytes for PreparedEvaluation {
    fn additional_charged_bytes(&self) -> u64 {
        self.retained_heap_charge(&mut Preparation::new(usize::MAX))
            .map_or(u64::MAX, Charge::bytes)
    }
}

impl RetainedStorageMeasurement for PreparedEvaluation {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.checked_result_heap_charge(work)?
            .checked_add(self.dependencies.retained_heap_charge(work)?)
    }
}

impl PreparedEvaluation {
    /// The checked callback's declared result heap excludes Signal's bounded
    /// dependency capture, which is admitted and measured separately.
    pub(crate) fn checked_result_heap_charge(
        &self,
        work: &mut Preparation,
    ) -> Result<Charge, Denial> {
        self.result
            .retained_heap_charge(work)?
            .checked_add(self.trace_data.retained_heap_charge(work)?)?
            .checked_add(self.keyed.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for NodeEvaluationResult {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.output_identity
            .retained_heap_charge(work)?
            .checked_add(self.continuity_token.retained_heap_charge(work)?)?
            .checked_add(self.changed_regions.retained_heap_charge(work)?)?
            .checked_add(self.changed_aspect_regions.retained_heap_charge(work)?)?
            .checked_add(self.labels.retained_heap_charge(work)?)
    }
}

impl RetainedStorageMeasurement for PreparedDependencyCapture {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        self.edges.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for super::capture::PreparedDependencyEdge {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.scope.retained_heap_charge(work)
    }
}

impl RetainedStorageMeasurement for PreparedTraceData {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        let temporal = match &self.temporal_eligibility {
            Some(temporal) => temporal.condition().retained_heap_charge(work)?,
            None => Charge::ZERO,
        };
        self.labels
            .retained_heap_charge(work)?
            .checked_add(self.causality.retained_heap_charge(work)?)?
            .checked_add(temporal)
    }
}

impl RetainedStorageMeasurement for PreparedKeyedContext {
    fn retained_heap_charge(&self, work: &mut Preparation) -> Result<Charge, Denial> {
        work.visit()?;
        self.family
            .retained_heap_charge(work)?
            .checked_add(self.key.retained_heap_charge(work)?)?
            .checked_add(self.memo_key.retained_heap_charge(work)?)?
            .checked_add(self.persistent_correspondence.retained_heap_charge(work)?)?
            .checked_add(self.composition_regions.retained_heap_charge(work)?)
    }
}
