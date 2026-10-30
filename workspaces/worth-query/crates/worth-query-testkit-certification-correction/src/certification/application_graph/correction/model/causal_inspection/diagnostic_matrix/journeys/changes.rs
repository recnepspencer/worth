use super::super::super::super::{edit::Value, other_edit::OtherEdit};
use super::super::super::absence;
use super::*;
pub fn optional_presence_and_nested_values_are_original_output_changes() {
    let application = distinct_subjects::install();
    distinct_subjects::link(&application, "numeric", "numeric", "matrix-nested-link");
    distinct_subjects::link(&application, "numeric", "observer", "matrix-second-link");
    drop(journey::perform(&application, 11, "matrix-initial-eleven"));
    drop(absence::perform(
        &application,
        Some(0),
        "matrix-present-operand",
        1,
    ));
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let access = || journey::access(&application, &principal, &scope);
    let request = application.runtime().request(&principal, &scope);
    let (before, rows) = read!(application, &principal, &scope, Read, true, true);
    assert_eq!(rows[0].operand, Some(0));
    assert_eq!(rows[0].references, vec![(11, 23), (17, 0)]);
    drop(absence::perform(
        &application,
        None,
        "matrix-clear-operand",
        1,
    ));
    let (absent, rows) = read!(application, &principal, &scope, Read, true, true);
    assert_eq!(rows[0].operand, None);
    assert_eq!(rows[0].references, vec![(11, 23), (17, 0)]);
    let report = request
        .inspect::<Binding>(&absent)
        .why_changed(&before)
        .reference_only()
        .execute_governed(access(), access())
        .unwrap();
    assert_eq!(report.reason(), CausalInspectionReason::ChangedResult);
    report.close();
    let outcome = request
        .advanced()
        .mutate(OtherEdit {
            key: "numeric".into(),
            value: Value::Primary(33),
        })
        .without_source()
        .idempotency(&"matrix-reorder-nested".to_owned())
        .execute_performed::<super::super::super::super::output_program::OutputProgram, super::super::super::super::output_program::Root>(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome else {
        panic!("the real reorder source must perform");
    };
    drop(journey::settle_source(&application, &request, performed, 1));
    let (nested, rows) = read!(application, &principal, &scope, Read, true, true);
    assert_eq!(rows[0].operand, None);
    assert_eq!(rows[0].references, vec![(17, 0), (33, 67)]);
    let report = request
        .inspect::<Binding>(&nested)
        .why_changed(&absent)
        .reference_only()
        .execute_governed(access(), access())
        .unwrap();
    assert_eq!(report.reason(), CausalInspectionReason::ChangedResult);
    report.close();
    before.close();
    absent.close();
    nested.close();
    assert_eq!(readers.observe().active(), baseline);
}
