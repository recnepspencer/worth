//! Real native case transitions, original absence, recorded reversal and explicit reapplication.
use super::*;

#[test]
fn changed_optional_cases_reverse_and_reapply_without_capturing_unchanged_fields() {
    let host = world::install_with_neighbour();
    let initial = query::Row {
        class: 1,
        first: Some(11),
        second: None,
        required: None,
        third: None,
    };
    let expected = query::Row {
        class: 2,
        first: None,
        second: Some(42),
        required: Some(420),
        third: None,
    };
    assert_eq!(world::read(&host), initial);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let neighbour = || {
        request
            .query(query::Read {
                key: "neighbour".into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .clone()
    };
    let untouched = query::Row {
        class: 1,
        first: Some(999),
        second: None,
        required: None,
        third: None,
    };
    assert_eq!(neighbour(), untouched);
    let outcome = request
        .advanced()
        .mutate(Input {
            key: "record".into(),
            case: Case::Second {
                value: 42,
                required: 420,
            },
        })
        .without_source()
        .idempotency(&"sparse-source".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: source, ..
    } = outcome
    else {
        panic!("sparse source: {outcome:?}");
    };
    assert_eq!(world::read(&host), expected);
    assert_eq!(neighbour(), untouched);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&source, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("sparse-undo")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("sparse inverse must prepare");
    };
    let outcome = prepared.publish().unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: reversal, ..
    } = outcome
    else {
        panic!("sparse inverse: {outcome:?}");
    };
    assert_eq!(world::read(&host), initial);
    assert_eq!(neighbour(), untouched);
    let selected = request.corrections().select(&source, limits).unwrap();
    let page = request
        .corrections()
        .alternatives(&selected, limits, std::num::NonZeroUsize::MIN)
        .unwrap();
    assert!(page.continuation().is_none());
    assert_eq!(page.entries().len(), 1);
    assert_eq!(
        page.entries()[0].performed_commit(),
        reversal.commit_reference()
    );
    let alternative = page.select(&page.entries()[0], limits).unwrap();
    let prepared = request
        .corrections()
        .reapply(&alternative)
        .idempotency("sparse-reapply")
        .limits(limits)
        .prepare()
        .unwrap();
    let WorthQueryApplicationCorrectionPreparation::Prepared(prepared) = prepared else {
        panic!("sparse reapplication must prepare");
    };
    let outcome = prepared.publish().unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    assert_eq!(world::read(&host), expected);
    assert_eq!(neighbour(), untouched);
}
