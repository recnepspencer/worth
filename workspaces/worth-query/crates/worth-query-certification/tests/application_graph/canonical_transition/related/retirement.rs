//! Ordinary precise retirement preserves already owned historical results.
use super::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::{application_entry::*, runtime::ExecutionAllocationPolicy};
#[test]
fn actual_v2_three_role_retirement_unlinks_only_observed_edges_and_reopens_current_source() {
    let original = host::install(model::authored_value()).unwrap();
    let _old = host::produce(&original);
    let target = journey::successor(host::capture(&original));
    let _new = successor::host::produce(&target);
    journey::assert_target_pages(&target, false);
    let source_id = successor::host::source_id(&target);
    let owned = successor::host::read_derived(&target);
    let scope = request_scope();
    let principal = authenticate_operator(target.installed_schema(), &scope);
    let request = target
        .runtime()
        .request(&principal, &scope)
        .on_branch(target.runtime().current_world());
    let observed = request
        .query(successor::query::ReadValue {
            identity: "authored-nested".into(),
        })
        .execute()
        .unwrap();
    let outcome = request
        .advanced()
        .mutate(successor::retire::Input {
            identity: "authored-nested".into(),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&180601)
        .execute_in_program(&target, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("precise native retirement must publish")
    };
    let roles = receipt
        .outputs_of::<successor::output::RetiredOutputs>()
        .unwrap();
    assert!(roles.entity::<successor::output::RetiredOutput>().is_ok());
    assert!(roles.entity::<successor::output::RetiredLeft>().is_ok());
    assert!(roles.entity::<successor::output::RetiredRight>().is_ok());
    assert!(request
        .query(successor::page_query::ReadPage {
            identity: "derived-report".into()
        })
        .page(
            std::num::NonZeroUsize::MIN,
            std::num::NonZeroUsize::new(65_536).unwrap()
        )
        .is_err());
    assert_eq!(successor::output::literals(&owned).1, 1012);
    assert_eq!(successor::host::read(&target), model::authored_value());
    let checkpoint = successor::host::capture(&target);
    drop(target);
    let reopened = successor::host::restore(checkpoint);
    assert_eq!(successor::host::source_id(&reopened), source_id);
    assert_eq!(successor::host::read(&reopened), model::authored_value());
    assert_eq!(successor::output::literals(&owned).1, 1012);
}
