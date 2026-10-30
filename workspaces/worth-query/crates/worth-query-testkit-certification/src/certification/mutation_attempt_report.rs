//! Real installed publication attempts: work belongs to an attempt, not its receipt.
pub use super::amendment;
pub use super::product_workflow_support::{self, principal, read_input, ExampleApplication};
pub use product_workflow_support::application_entry::AmendTemporalDenial;
pub use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt, WorthQueryApplicationRequestMutationDenial as Denial,
    },
    primary_graph::{WorthQueryInvariantProjectionWork, WorthQueryMutationHandlerWork},
};

pub fn captured(work: WorthQueryMutationHandlerWork) -> WorthQueryInvariantProjectionWork {
    let WorthQueryMutationHandlerWork::Captured(capture) = work else {
        panic!("the installed decision must produce an execution-owned capture")
    };
    assert!(capture.handler_contacted());
    let projection = capture.projection_work();
    assert!(projection.provider_work_units() > 0);
    assert!(projection.field_reads() > 0);
    projection
}
