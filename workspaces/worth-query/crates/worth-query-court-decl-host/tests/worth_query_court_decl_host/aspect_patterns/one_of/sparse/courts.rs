//! Real native case transitions, original absence, recorded reversal and explicit reapplication.
use super::*;

#[test]
fn changed_optional_cases_reverse_and_reapply_without_capturing_unchanged_fields() {
    exercise_case_inverse(false, false);
}

#[test]
fn changed_fields_reverse_and_reapply_while_a_nonempty_relation_is_unchanged() {
    exercise_case_inverse(true, false);
}

#[test]
fn authenticated_absence_alone_closes_zero_relation_changes() {
    exercise_case_inverse(true, true);
}
fn exercise_case_inverse(keep_link: bool, annotate: bool) {
    let host = world::install_with_neighbour();
    let initial = query::Row {
        class: 1,
        first: Some(11),
        second: None,
        required: None,
        third: None,
    };
    let expected = if annotate {
        initial.clone()
    } else {
        query::Row {
            class: 2,
            first: None,
            second: Some(42),
            required: Some(420),
            third: None,
        }
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
        first: Some(99),
        second: None,
        required: None,
        third: None,
    };
    assert_eq!(neighbour(), untouched);
    let links = || {
        request
            .query(link_query::Read {
                key: "record".into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .targets
            .clone()
    };
    let annotation = || {
        request
            .query(link_query::Read {
                key: "record".into(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .annotation
    };
    assert_eq!(annotation(), None);
    assert!(links().is_empty());
    if keep_link {
        let outcome = request
            .advanced()
            .mutate(Input {
                key: "record".into(),
                case: Case::First { value: 11 },
                link: LinkEdit::Set(LinkTarget::Named("neighbour".into())),
                observation_fault: ObservationFault::None,
            })
            .without_source()
            .idempotency(&"establish-companion".to_owned())
            .execute_in_program(
                &host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{outcome:?}"
        );
        assert_eq!(links(), ["neighbour"]);
    }
    let outcome = request
        .advanced()
        .mutate(Input {
            key: "record".into(),
            case: if annotate {
                Case::Annotation { value: Some(7) }
            } else {
                Case::Second {
                    value: 42,
                    required: 420,
                }
            },
            link: LinkEdit::Set(LinkTarget::Named("neighbour".into())),
            observation_fault: ObservationFault::None,
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
    assert_eq!(annotation(), annotate.then_some(7));
    assert_eq!(links(), ["neighbour"]);
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
    assert_eq!(annotation(), None);
    if keep_link {
        assert_eq!(links(), ["neighbour"]);
    } else {
        assert!(links().is_empty());
    }
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
    assert_eq!(annotation(), annotate.then_some(7));
    assert_eq!(links(), ["neighbour"]);
    assert_eq!(neighbour(), untouched);
}

#[test]
fn missing_original_or_same_kind_neighbour_never_supplies_a_changed_output_preimage() {
    for fault in [ObservationFault::Unobserved, ObservationFault::ForeignOnly] {
        let host = world::install_with_neighbour();
        let before = world::read(&host);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.runtime().request(&principal, &scope);
        let pinned = request.retain_current_read().unwrap();
        let outcome = request
            .advanced()
            .mutate(Input {
                key: "record".into(),
                case: Case::Second {
                    value: 42,
                    required: 420,
                },
                link: LinkEdit::Set(LinkTarget::Named("neighbour".into())),
                observation_fault: fault,
            })
            .without_source()
            .idempotency(&"incomplete-original".to_owned())
            .execute_in_program(
                &host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        let denial = outcome.unwrap_err();
        let WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Handler(denial),
        ) = denial
        else {
            panic!("{denial:?}")
        };
        let FragmentDenial::Write(denial) = denial.downcast::<FragmentDenial>().unwrap() else {
            panic!("actual native write refusal")
        };
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::UnobservedWrite
        );
        let cause = denial.unobserved_write().unwrap();
        assert_eq!(
            cause.locator().aspect().aspect_key().as_str(),
            SecondValue::at("SecondValue").aspect()
        );
        assert_eq!(
            cause.locator().field_path().fields()[0].as_str(),
            "SecondValue"
        );
        assert_eq!(
            pinned.selected_commit(),
            request.retain_current_read().unwrap().selected_commit()
        );
        assert_eq!(world::read(&host), before);
        assert!(request
            .query(link_query::Read {
                key: "record".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .targets
            .is_empty());
    }
}
