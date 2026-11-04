//! selection denies invalid work and overflow budgets without retaining partial results.
use crate::domain_computation::primary_graph::application_attempt::read_set::source_facts::indexed_selection::*;
use worth_query_execution::facade::domain_computation::WorthQueryEntityResolutionRefusalKind as ResolutionRefusal;

#[test]
fn selection_denies_invalid_work_and_overflow_budgets_without_retaining_partial_results() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let label = AccountLabel::reference();
    world
        .application
        .with_host_advancement(|phase| {
            set_unrelated_value(
                &phase,
                &world,
                "primary",
                (label.entity(), label.aspect(), label.field()),
            )
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request_for_test(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let actor = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &actor,
                    &account,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let (_, projection, work) = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, _| {
                        for (limit, expected) in [
                            (
                                0,
                                WorthQueryEntityResolutionDenialKind::Refused(
                                    ResolutionRefusal::InvalidCandidateLimit,
                                ),
                            ),
                            (
                                usize::MAX,
                                WorthQueryEntityResolutionDenialKind::Refused(
                                    ResolutionRefusal::InvalidCandidateLimit,
                                ),
                            ),
                            (
                                1,
                                WorthQueryEntityResolutionDenialKind::Refused(
                                    ResolutionRefusal::CandidateLimitExceeded { maximum: 1 },
                                ),
                            ),
                        ] {
                            let denial = reader
                                .decision_select_entities(
                                    AccountLabel::reference(),
                                    "primary".to_owned(),
                                    limit,
                                )
                                .unwrap_err()
                                .downcast::<WorthQueryEntityResolutionDenial>()
                                .unwrap();
                            assert_eq!(denial.kind(), expected);
                        }
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        work.equality_lookups(),
        1,
        "invalid budgets deny before index work"
    );
    assert_eq!(work.index_candidates_examined(), 1);
    let reads = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .begin_projected_application_read_attempt(
                    &phase,
                    admission,
                    projection,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .complete_projected_dependencies(
                    &phase,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert!(
        !(*reads
            .facts_for_test())
            .iter()
            .any(|fact| matches!(fact, Fact::IndexedEntitySelection { .. })),
        "an overflowing prefix cannot become a complete source fact"
    );
}
