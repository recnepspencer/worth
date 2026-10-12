use crate::intent_admission::certification_runtime;
use crate::runtime::{WorthQueryAspectMutationBuilder, WorthQueryWriteReceipt};

use super::{status_value_touch, title_value_touch};

pub(super) fn certification_query_write_receipt(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryWriteReceipt {
    let mut workspace = certification_runtime(execution)
        .workspace("lower-runtime-projection-query-receipts")
        .expect("projection query receipt workspace should build");
    workspace
        .insert(
            execution,
            "Task",
            |task: WorthQueryAspectMutationBuilder| {
                task.set_aspect(
                    title_value_touch(),
                    crate::runtime::WorthQueryAuthoredAspectValue::string("Projection fixture"),
                )
                .set_aspect(
                    status_value_touch(),
                    crate::runtime::WorthQueryAuthoredAspectValue::string("todo"),
                )
            },
        )
        .expect("projection query receipt write should execute")
}
