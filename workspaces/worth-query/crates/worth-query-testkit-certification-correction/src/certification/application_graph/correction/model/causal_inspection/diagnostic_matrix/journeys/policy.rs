use super::super::super::super::query::full;
use super::super::super::query as lawful_query;
use super::*;
pub fn current_policy_and_original_parameters_precede_body_availability() {
    let application = host::install_with_outputs();
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let readers = application.application_query_basis_observer();
    let baseline = readers.observe().active();
    let access = || journey::access(&application, &principal, &scope);
    let request = application.runtime().request(&principal, &scope);
    macro_rules! denied {
        ($read:ident, $binding:ident) => {{
            let (before, rows) = read!(application, &principal, &scope, $read, false, false);
            assert!(rows.is_empty());
            let (after, rows) = read!(application, &principal, &scope, $read, false, false);
            assert!(rows.is_empty());
            let denial = request
                .inspect::<$binding>(&after)
                .why_changed(&before)
                .reference_only()
                .execute_governed(access(), access())
                .err()
                .expect("current explanation policy refuses before absent bodies");
            assert_eq!(
                denial.kind(),
                WorthQueryApplicationInspectionStopKind::Admission
            );
            assert!(denial.materialization_refusal().is_none());
            drop(denial);
            before.close();
            after.close();
        }};
    }
    denied!(InternalRead, InternalBinding);
    denied!(RelationRead, RelationBinding);
    denied!(FieldRead, FieldBinding);
    let (before, rows) = read!(application, &principal, &scope, Read, false, true);
    assert!(rows.is_empty());
    let (after, rows) = read!(application, &principal, &scope, Read, false, true);
    assert!(rows.is_empty());
    let report = request
        .inspect::<Binding>(&after)
        .why_changed(&before)
        .reference_only()
        .execute_governed(access(), access())
        .unwrap();
    assert_eq!(report.reason(), CausalInspectionReason::UnchangedResult);
    report.close();
    before.close();
    after.close();
    let (before, rows) = read!(application, &principal, &scope, Read, true, false);
    assert_eq!(rows.len(), 1);
    let (after, rows) = read!(application, &principal, &scope, Read, true, false);
    assert_eq!(rows.len(), 1);
    let denied = request
        .inspect::<Binding>(&after)
        .why_changed(&before)
        .reference_only()
        .execute_governed(access(), access())
        .err()
        .expect("equal parameters reach absent-body refusal");
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Unavailable
    );
    drop(denied);
    let (different, rows) = read!(application, &principal, &scope, Read, false, false);
    assert!(rows.is_empty());
    let denied = request
        .inspect::<Binding>(&different)
        .why_changed(&before)
        .reference_only()
        .execute_governed(access(), access())
        .err()
        .expect("actual parameter mismatch precedes both absent bodies");
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    drop(denied);
    let result = request
        .query(lawful_query::Read {
            key: "numeric".into(),
        })
        .execute_governed(
            capability::Inspect::reference(),
            capability::InspectCausalEvidence::reference(),
            capability::Input::at("numeric"),
        )
        .unwrap();
    let foreign_binding = result.receipt().inspect().observation().unwrap().clone();
    drop(result);
    let lawful = request
        .inspect::<lawful_query::Binding>(&foreign_binding)
        .why_changed(&foreign_binding)
        .reference_only()
        .execute_governed(access(), access())
        .err()
        .expect("lawful foreign binding reaches missing-body lookup at its own identity");
    assert_eq!(
        lawful.kind(),
        WorthQueryApplicationInspectionStopKind::Unavailable
    );
    drop(lawful);
    let denied = request
        .inspect::<Binding>(&after)
        .why_changed(&foreign_binding)
        .reference_only()
        .execute_governed(access(), access())
        .err()
        .expect("a genuine different binding cannot lend an original");
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    drop(denied);
    let result = request
        .query(full::Read {
            key: "numeric".into(),
        })
        .retain_projection_for_inspection()
        .execute()
        .unwrap();
    let public = result.receipt().inspect().observation().unwrap().clone();
    drop(result);
    let denied = request
        .inspect::<full::Binding>(&public)
        .why_changed(&public)
        .reference_only()
        .execute_with_diagnostic(access())
        .err()
        .expect("retained public payload and metadata do not declare diagnostic comparison");
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Unavailable
    );
    drop(denied);
    public.close();
    let stale_metadata = access();
    let stale_diagnostic = access();
    let outcome = request
        .advanced()
        .mutate(distinct_subjects::grant_status::Withdraw {
            grant: "inspection-grant".into(),
        })
        .without_source()
        .idempotency(&"matrix-withdraw-inspection".to_owned())
        .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    let denied = request
        .inspect::<Binding>(&different)
        .why_changed(&different)
        .reference_only()
        .execute_governed(stale_metadata, stale_diagnostic)
        .err()
        .expect("withdrawn current grants refuse before absent empty bodies");
    assert_eq!(
        denied.kind(),
        WorthQueryApplicationInspectionStopKind::Admission
    );
    assert!(denied.materialization_refusal().is_none());
    drop(denied);
    before.close();
    after.close();
    different.close();
    foreign_binding.close();
    assert_eq!(readers.observe().active(), baseline);
}
