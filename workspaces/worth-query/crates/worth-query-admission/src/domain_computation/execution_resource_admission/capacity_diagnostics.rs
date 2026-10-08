//! Required present dimensions are compared without dense accessor assumptions.
use super::WorthQueryExecutionResourceSupport;
use worth_query_declaration::facade::domain_computation::WorthQuerySemanticScaleAxis;
use worth_query_installation::facade::WorthQueryExecutionStrategyContract;
pub(super) fn capacity_mismatch_detail(
    subject: &str,
    strategy: &WorthQueryExecutionStrategyContract,
    actual: &WorthQueryExecutionResourceSupport,
) -> String {
    for (axis, required) in strategy.envelope().scale_ceilings().iter() {
        let Some(supported) = actual.envelope().optional_scale_ceiling(axis) else {
            if axis == WorthQuerySemanticScaleAxis::WorkItems {
                continue;
            }
            return format!("{subject} does not support required {axis:?}");
        };
        if supported < required {
            return format!(
                "{subject} supports {axis:?}={supported}, below the required {required}"
            );
        }
    }
    for (dimension, required) in strategy.envelope().resource_ceilings().iter() {
        let Some(supported) = actual.envelope().optional_resource_ceiling(dimension) else {
            return format!("{subject} does not support required {dimension:?}");
        };
        if supported < required {
            return format!(
                "{subject} supports {dimension:?}={supported}, below the required {required}"
            );
        }
    }
    format!("{subject} capacity changed during resource admission")
}
