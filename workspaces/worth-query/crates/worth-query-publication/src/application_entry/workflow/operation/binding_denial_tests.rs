//! Binding errors keep their original native owner in the public error chain.
use super::*;
use std::error::Error;
#[test]
fn binding_denial_retains_advancement_and_original_actor_causes() {
    use worth_query_execution::facade::{
        application_contribution::WorthQueryAdvancementDenial,
        primary_graph::WorthQueryApplicationAttemptDenial,
    };
    let opening = WorthQueryAdvancementDenial::NestedOpening;
    let denial = WorthQueryWorkflowOperationBindingDenial::ExecutionRequest(opening);
    assert_eq!(
        denial
            .source()
            .unwrap()
            .downcast_ref::<WorthQueryAdvancementDenial>(),
        Some(&opening)
    );
    let cause = Box::new(WorthQueryApplicationAttemptDenial::from(
        worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
    ));
    let original = cause.as_ref() as *const WorthQueryApplicationAttemptDenial;
    let denial = WorthQueryWorkflowOperationBindingDenial::ActorContext(cause);
    let retained = denial
        .source()
        .unwrap()
        .downcast_ref::<WorthQueryApplicationAttemptDenial>()
        .unwrap();
    assert!(std::ptr::eq(original, retained));
}
