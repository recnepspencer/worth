//! The fixture declares the complete heap shape its checked evaluator can
//! return. The compiler owns this model; the Signal planner only sees bytes.

use crate::data::aspect::AspectVersion;
use crate::data::error::SignalError;
use crate::data::output::{ChangedRegion, NodeEvaluationResult};
use crate::data::retained_storage::{RetainedStorageMeasurement, RetainedStoragePreparation};

use super::super::super::{
    FinancialLocalityAction, FinancialLocalityDefinition, FinancialLocalityOutput,
};
use super::super::topology::signal_aspect;

pub(super) fn maximum_result_heap(
    definition: &FinancialLocalityDefinition,
    output: &FinancialLocalityOutput,
) -> Result<u64, SignalError> {
    // One evaluation program receives one action trace's mutation batch (or
    // a subset of it). Traces run separately, so their region vectors never
    // coexist in a returned result. The empty batch also covers bootstrap.
    // Both i64 extremes cover the longest formatted financial identity.
    let mut maximum = 0;
    for trace in std::iter::once(None).chain(definition.action_traces().iter().map(Some)) {
        for value in [i64::MIN, i64::MAX] {
            let mut result = NodeEvaluationResult::from_version(AspectVersion::zero())
                .with_output_identity(format!(
                    "financial-locality:{:?}:{:?}:{}:{}",
                    output.owner,
                    output.role,
                    output.id.ordinal(),
                    value,
                ));
            if let Some(trace) = trace {
                for action in trace.actions() {
                    let FinancialLocalityAction::CommitFactor(mutation) = action else {
                        continue;
                    };
                    if mutation.producer != output.id {
                        continue;
                    }
                    if let Some(scope) = mutation.scope {
                        let mut region = ChangedRegion::new(scope.partition_label());
                        if let Some(detail) = scope.detail_label() {
                            region = region.with_detail(detail);
                        }
                        result = result
                            .with_changed_aspect_region(signal_aspect(mutation.aspect), region);
                    }
                }
            }
            let candidate = result
                .retained_heap_charge(&mut RetainedStoragePreparation::new(usize::MAX))
                .map(|charge| charge.bytes())
                .map_err(|_| SignalError::EvaluationStorageCapacityExhausted)?;
            maximum = maximum.max(candidate);
        }
    }
    Ok(maximum)
}
