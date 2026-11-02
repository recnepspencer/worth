//! The closed physical fresh route admits the SAME original Native owner.
use super::RelationalSectionReadmissionDenial as Denial;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::FreshRelationalSourceAdmission, runtime::RelationalRuntime,
};

pub(in crate::section) fn admit(
    runtime: &mut RelationalRuntime,
    source: &Arc<crate::cold_source::StoreColdSource>,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<FreshRelationalSourceAdmission, Denial> {
    execution.consult().map_err(Denial::Execution)?;
    let mut result = None;
    let scope = execution.in_scope(|_| {
        result = Some((|| {
            let plan = super::prepare_source_plan(runtime, source, None, execution)?;
            runtime
                .durability_recovery()
                .admit_fresh_home_source(execution, plan)
                .map_err(Denial::Native)
        })());
    });
    match (scope, result) {
        (_, Some(result)) => result,
        (Err(cause), None) => Err(Denial::Execution(cause)),
        (Ok(()), None) => unreachable!("completed original-request source admission"),
    }
}
