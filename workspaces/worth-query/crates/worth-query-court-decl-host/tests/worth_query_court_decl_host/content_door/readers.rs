//! Public byte readers share installed admission and field dependencies.
use super::*;
pub mod computation;
pub mod invariant;
use std::sync::{atomic::Ordering, Arc};

#[test]
fn content_readers_resolve_identical_bytes_and_follow_field_identity_changes() {
    let calls = Arc::new(source::Calls::default());
    let (host, projection) = world::install_with_projection(None, calls.clone());
    for (sample, key) in [(7, "seven"), (23, "twenty-three")] {
        assert!(matches!(
            world::edit(&host, 0, vec![sample], vec![], key),
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        let expected = world::observe(&host, &format!("observe-{key}")).unwrap();
        assert!(
            calls.reader_bytes.lock().unwrap().contains(&expected.1),
            "the proposed invariant view reads the newly staged identity"
        );
        host.runtime().with_host_advancement(|phase| {
            let ordinary = projection.project(&phase, |reader| {
                let entity = reader.resolve_entity(Key::at("Key"), "record".to_owned()).unwrap();
                let content = reader.content(&entity, samples()).unwrap().unwrap();
                (content.identity(), content.payload().to_vec())
            }, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation).unwrap();
            assert_eq!(ordinary.output(), &expected);
            let operation = projection.project_operation::<operation::Change, _>(&phase, |reader| {
                let entity = reader.resolve_entity(Key::at("Key"), "record".to_owned()).unwrap();
                let content = reader.content(&entity, samples()).unwrap().unwrap();
                (content.identity(), content.payload().to_vec())
            }, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation).unwrap();
            assert_eq!(operation.output(), &expected);
        }).unwrap();
        let computed = world::edit(&host, 30, vec![], vec![], &format!("compute-{key}"));
        assert_eq!(
            computed.result().unwrap().before.as_ref().unwrap(),
            &expected
        );
    }
}

#[test]
fn content_projection_readers_refuse_a_substituted_format_even_if_swallowed() {
    let (host, projection) = world::install_with_projection(None, Default::default());
    world::edit(&host, 0, vec![7], vec![], "seed");
    host.runtime()
        .with_host_advancement(|phase| {
            let ordinary = projection.project(
                &phase,
                |reader| {
                    let entity = reader
                        .resolve_entity(Key::at("Key"), "record".to_owned())
                        .unwrap();
                    assert!(matches!(
                        reader.content(
                            &entity,
                            content_field(Values::at("Values"), codec::Impostor)
                        ),
                        Err(ContentReadDenial::BindingContractMismatch)
                    ));
                },
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
            assert_eq!(
                ordinary.err().unwrap().kind(),
                WorthQueryInvariantProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::BindingContractMismatch
                )
            );
            let operation = projection.project_operation::<operation::Change, _>(
                &phase,
                |reader| {
                    let entity = reader
                        .resolve_entity(Key::at("Key"), "record".to_owned())
                        .unwrap();
                    assert!(matches!(
                        reader.content(
                            &entity,
                            content_field(Values::at("Values"), codec::Impostor)
                        ),
                        Err(ContentReadDenial::BindingContractMismatch)
                    ));
                },
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
            assert_eq!(
                operation.err().unwrap().kind(),
                WorthQueryInvariantProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::BindingContractMismatch
                )
            );
        })
        .unwrap();
}

#[test]
fn absent_optional_content_is_absence_on_projection_and_invariant_readers() {
    let calls = Arc::new(source::Calls::default());
    let (host, projection) = world::install_with_projection(None, calls.clone());
    host.runtime()
        .with_host_advancement(|phase| {
            projection
                .project(
                    &phase,
                    |reader| {
                        let entity = reader
                            .resolve_entity(Key::at("Key"), "record".to_owned())
                            .unwrap();
                        assert!(reader.content(&entity, samples()).unwrap().is_none());
                    },
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
        })
        .unwrap();
    assert!(calls.reader_bytes.lock().unwrap().is_empty());
    assert_eq!(calls.content_reads.load(Ordering::SeqCst), 0);
}

#[test]
fn cancelled_content_projection_releases_its_output_and_a_fresh_read_succeeds() {
    let (host, projection) = world::install_with_projection(None, Default::default());
    world::edit(&host, 0, vec![7], vec![], "seed-cancel");
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        WorthQueryAdmissionClock::system(),
    );
    let selected = world::pin(&host);
    let stopped =
        host.runtime()
            .with_application_advancement(&scope, |phase| {
                let execution = phase
                    .checked_execution_request_for(host.runtime().product_runtime())
                    .unwrap();
                execution.in_scope(|lease| {
                    let payer = lease.expect("the host lends its original funded lease");
                    let held = payer.charged_memory_bytes();
                    let result = projection.project(&phase, |reader| {
                let entity = reader.resolve_entity(Key::at("Key"), "record".to_owned()).unwrap();
                let content = reader.content(&entity, samples()).unwrap().unwrap();
                cancellation.cancel();
                content
            }, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation);
                    assert!(result.is_err(), "cancelled custody cannot escape");
                    assert_eq!(
                        payer.charged_memory_bytes(),
                        held,
                        "refused projection releases its original-funded read custody"
                    );
                    result
                })
            });
    assert!(
        matches!(stopped, Err(_) | Ok(Err(_)) | Ok(Ok(Err(_)))),
        "cancelled custody cannot escape projection completion"
    );
    assert_eq!(
        world::pin(&host).selected_commit(),
        selected.selected_commit()
    );
    assert_eq!(
        world::observe(&host, "fresh-after-cancel").unwrap().1,
        [1, 0, 0, 0, 1, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0]
    );
}

#[test]
fn malformed_and_missing_content_refuse_before_any_reader_can_publish() {
    let calls = Arc::new(source::Calls::default());
    let host = world::install_with_calls(None, calls.clone());
    world::edit(&host, 0, vec![7], vec![], "seed-refusals");
    let selected = world::pin(&host);
    let observations = calls.reader_bytes.lock().unwrap().len();
    let missing = world::edit(&host, 2, vec![], vec![0xe7; 32], "dangling");
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        missing,
    )) = missing
    else {
        panic!("dangling reference published");
    };
    assert_eq!(
        missing.content_denial(),
        Some(RelationalContentDenial::MissingContent)
    );
    let corrupt = world::try_edit(&host, 8, vec![11], vec![], "corrupt").unwrap_err();
    assert!(matches!(
        corrupt,
        WorthQueryApplicationRequestMutationDenial::Handler(_)
    ));
    assert_eq!(
        world::pin(&host).selected_commit(),
        selected.selected_commit()
    );
    assert_eq!(calls.reader_bytes.lock().unwrap().len(), observations);
}

#[test]
fn invariant_content_format_refusal_cannot_be_swallowed_into_a_pass() {
    let calls = Arc::new(source::Calls::default());
    let host = world::install_with_calls(None, calls.clone());
    world::edit(&host, 0, vec![7], vec![], "invariant-seed");
    let selected = world::pin(&host);
    calls.reader_impostor.store(1, Ordering::SeqCst);
    let result = world::edit(&host, 0, vec![23], vec![], "swallowed-invariant-format");
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("caught content refusal published");
    };
    let cause = denial
        .custom_invariant_denial()
        .expect("the invariant reader refused");
    assert!(matches!(
        cause,
        WorthQueryCustomInvariantDenial::Failure { cause: Some(_), .. }
    ));
    assert_eq!(
        world::pin(&host).selected_commit(),
        selected.selected_commit()
    );
    calls.reader_impostor.store(0, Ordering::SeqCst);
    assert_eq!(world::observe(&host, "invariant-fresh").unwrap().1[8], 7);
}

#[test]
fn computation_content_format_refusal_cannot_be_swallowed_by_gather() {
    let host = world::install();
    world::edit(&host, 0, vec![7], vec![], "computation-seed");
    let selected = world::pin(&host);
    let refused =
        world::try_edit(&host, 31, vec![], vec![], "swallowed-gather-format").unwrap_err();
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Projection(cause),
    ) = refused
    else {
        panic!("wrong refusal: {refused:?}");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOperationProjectionDenialKind::InvariantAdmission(
            WorthQueryInvariantProjectionDenialKind::ValueDecode(
                WorthQueryValueDecodeDenial::BindingContractMismatch
            )
        )
    );
    assert_eq!(
        world::pin(&host).selected_commit(),
        selected.selected_commit()
    );
}

#[test]
fn operation_content_reader_refuses_a_field_outside_its_declared_reads() {
    let host = world::install();
    world::edit(&host, 0, vec![7], vec![], "permission-seed");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let refused = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(created_content::Issue {
            key: "undeclared-content-read".into(),
            samples: vec![11],
        })
        .without_source()
        .idempotency(&"undeclared-content".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Handler(cause),
    ) = refused
    else {
        panic!("wrong refusal: {refused:?}");
    };
    let ContentReadDenial::Field(cause) = cause.downcast::<ContentReadDenial>().unwrap() else {
        panic!("the operation contract must refuse the field read");
    };
    assert_eq!(
        cause
            .downcast::<WorthQueryInvariantDecisionPlanDenial>()
            .unwrap()
            .kind(),
        WorthQueryInvariantDecisionPlanDenialKind::UndeclaredDecisionTarget
    );
}
