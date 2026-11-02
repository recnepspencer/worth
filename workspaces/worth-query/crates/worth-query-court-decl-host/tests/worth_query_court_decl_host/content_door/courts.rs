use super::*;
#[test]
fn content_public_door_preserves_exact_order_and_deduplicates_equal_payloads() {
    let host = world::install();
    assert!(world::observe(&host, "initial").is_none());
    assert!(matches!(
        world::edit(&host, 0, vec![7, -11, 23], vec![], "first"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let first = world::observe(&host, "observe-first").unwrap();
    // Literal bytes are independent of every production encoder, parser and digest routine.
    let expected = [
        1, 0, 0, 0, 3, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 245, 255, 255, 255, 255, 255, 255, 255, 23,
        0, 0, 0, 0, 0, 0, 0,
    ];
    assert_eq!(first.1, expected);
    // Python hashlib/OpenSSL over the literal WRTHCONT frame, not the writer.
    assert_eq!(
        first.0.to_string(),
        "74fd5973013eab98296db92f3c2702393ded5d60c4b46e1119b4d8f2c147b6cb"
    );
    assert!(matches!(
        world::edit(&host, 0, vec![7, -11, 23], vec![], "equal"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(world::observe(&host, "observe-equal").unwrap(), first);
    world::edit(&host, 0, vec![23, -11, 7], vec![], "reordered");
    assert_ne!(
        world::observe(&host, "observe-reordered").unwrap().0,
        first.0
    );
    assert!(matches!(
        world::edit(&host, 1, vec![7, -11, 23], vec![], "initialize"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let observed_created = world::edit(&host, 16, vec![], vec![], "observe-created");
    assert_eq!(
        observed_created.result().unwrap().before.as_ref(),
        Some(&first)
    );
}
#[test]
fn content_raw_missing_reference_is_refused_and_the_selected_value_stays_unchanged() {
    let host = world::install();
    world::edit(&host, 0, vec![7, -11, 23], vec![], "first");
    let before = world::observe(&host, "before").unwrap();
    let selected = world::pin(&host);
    let refused = world::edit(&host, 2, vec![], vec![0xE7; 32], "missing");
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = refused
    else {
        panic!("missing content must refuse: {refused:?}");
    };
    assert_eq!(
        denial.content_denial(),
        Some(RelationalContentDenial::MissingContent)
    );
    assert_eq!(
        world::pin(&host).selected_commit(),
        selected.selected_commit()
    );
    assert_eq!(world::observe(&host, "after").unwrap(), before);
}
#[test]
fn content_callback_refusal_cannot_be_swallowed_into_publication() {
    let host = world::install();
    let refused = world::try_edit(&host, 3, vec![], vec![], "swallowed-noncanonical");
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    ) = refused.unwrap_err()
    else {
        panic!("caught callback refusal must veto candidate completion");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::ValueDecode(WorthQueryValueDecodeDenial::Content(
            RelationalContentDenial::NonCanonicalContent
        ))
    );
    assert!(world::observe(&host, "after").is_none());
}

#[test]
fn content_other_format_and_other_version_refuse_at_the_same_publication_seal() {
    let host = world::install();
    world::edit(&host, 0, vec![7, -11, 23], vec![], "initial-grid");
    world::edit(&host, 4, vec![5], vec![], "other-format");
    let outcome = world::edit(&host, 5, vec![13], vec![], "other-version");
    let WorthQueryApplicationMutationOutcome::Committed { result, .. } = outcome else {
        panic!("second format must publish: {outcome:?}");
    };
    let other = result.other.unwrap();
    let outcome = world::edit(&host, 99, vec![], vec![], "version-observation");
    let version = outcome.result().unwrap().version.unwrap();
    for (index, reference, cause) in [
        (0, other, RelationalContentDenial::FormatMismatch),
        (1, version, RelationalContentDenial::UnsupportedVersion),
    ] {
        let before = world::pin(&host).selected_commit().clone();
        let refused = world::edit(
            &host,
            2,
            vec![],
            reference.0.to_vec(),
            &format!("swapped-{index}"),
        );
        let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
            denial,
        )) = refused
        else {
            panic!("foreign content must refuse: {refused:?}");
        };
        assert_eq!(denial.content_denial(), Some(cause));
        assert_eq!(world::pin(&host).selected_commit(), &before);
    }
}
#[test]
fn content_equal_descriptors_cannot_substitute_another_binding_type_at_either_door() {
    let host = world::install();
    world::edit(&host, 0, vec![7], vec![], "initial");
    for mode in [6, 7] {
        let before = world::pin(&host).selected_commit().clone();
        let outcome = world::try_edit(&host, mode, vec![11], vec![], &format!("impostor-{mode}"));
        let cause = outcome.unwrap_err();
        match (mode, cause) {
            (
                6,
                WorthQueryApplicationRequestMutationDenial::Handler(
                    MutationHandlerExecutionDenial::Projection(cause),
                ),
            ) => {
                assert_eq!(
                    cause.kind(),
                    WorthQueryOperationProjectionDenialKind::InvariantAdmission(
                        WorthQueryInvariantProjectionDenialKind::ValueDecode(
                            WorthQueryValueDecodeDenial::BindingContractMismatch
                        )
                    )
                );
            }
            (
                7,
                WorthQueryApplicationRequestMutationDenial::Handler(
                    MutationHandlerExecutionDenial::Attempt(cause),
                ),
            ) => {
                assert_eq!(
                    cause.kind(),
                    WorthQueryApplicationAttemptDenialKind::ValueDecode(
                        WorthQueryValueDecodeDenial::BindingContractMismatch
                    )
                );
            }
            (_, cause) => panic!("typed binding mismatch was lost: {cause:?}"),
        }
        assert_eq!(world::pin(&host).selected_commit(), &before);
    }
}

#[test]
fn content_duplicate_format_declarations_refuse_before_installation() {
    let duplicate = schema::declare(ApplicationSchemaDeclarationBuilder::<Schema>::for_schema())
        .content_format::<codec::GridFormat>()
        .build()
        .unwrap_err();
    assert_eq!(
        duplicate,
        ApplicationSchemaDeclarationDenial::DuplicateContentFormat
    );
}

#[test]
fn content_corrupted_callback_result_is_caught_by_independent_numeric_correspondence() {
    let host = world::install();
    let before = world::pin(&host).selected_commit().clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let refused = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(operation::Edit {
            mode: 8,
            samples: vec![7, -11, 23],
            reference: vec![],
        })
        .without_source()
        .idempotency(&"corrupt-result".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Handler(cause),
    ) = refused.unwrap_err()
    else {
        panic!("the content writer must retain its codec refusal");
    };
    assert!(matches!(
        cause.downcast::<ContentWriteDenial>().unwrap(),
        ContentWriteDenial::Content(RelationalContentDenial::EncodingInconsistency)
    ));
    assert_eq!(world::pin(&host).selected_commit(), &before);
}

#[test]
fn content_sink_refusal_cannot_be_swallowed_by_a_successful_callback() {
    let host = world::install();
    let before = world::pin(&host).selected_commit().clone();
    let cause = world::try_edit(&host, 9, vec![7], vec![], "swallowed-sink").unwrap_err();
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Handler(cause),
    ) = cause
    else {
        panic!("the content sink must retain its original refusal");
    };
    assert!(matches!(
        cause.downcast::<ContentWriteDenial>().unwrap(),
        ContentWriteDenial::Content(RelationalContentDenial::EncodingInconsistency)
    ));
    assert_eq!(world::pin(&host).selected_commit(), &before);
}

#[test]
fn content_shared_format_and_optional_field_clear_preserve_required_sibling_then_parent_removal() {
    let host = world::install();
    world::edit(&host, 0, vec![7, -11, 23], vec![], "required");
    let first = world::observe(&host, "required-observed").unwrap();
    assert!(matches!(
        world::edit(&host, 12, vec![7, -11, 23], vec![], "shared"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let shared = world::edit(&host, 99, vec![], vec![], "shared-observed");
    assert_eq!(shared.result().unwrap().optional, Some(first.0));
    world::edit(&host, 13, vec![], vec![], "clear-optional");
    let cleared = world::edit(&host, 99, vec![], vec![], "cleared-observed");
    assert_eq!(cleared.result().unwrap().optional, None);
    assert_eq!(cleared.result().unwrap().before.as_ref(), Some(&first));
    assert!(matches!(
        world::edit(&host, 11, vec![], vec![], "remove-parent"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert!(world::observe(&host, "absent-parent").is_none());
}

#[test]
fn content_original_cancellation_during_encoding_refuses_before_publication_then_fresh_request_succeeds(
) {
    let cancellation = WorthQueryCancellationSource::new();
    let host = world::install_with_control(Some(cancellation.clone()));
    let before = world::pin(&host).selected_commit().clone();
    let scope = WorthQueryRequestScope::new(None, cancellation.token());
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let refused = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(operation::Edit {
            mode: 14,
            samples: vec![7, -11, 23],
            reference: vec![],
        })
        .without_source()
        .idempotency(&"cancel-encoding".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    match refused {
        Ok(WorthQueryApplicationMutationOutcome::Cancelled) => {}
        Err(WorthQueryApplicationRequestMutationDenial::Handler(
            MutationHandlerExecutionDenial::Handler(cause),
        )) => {
            assert!(matches!(
                cause.downcast::<ContentWriteDenial>().unwrap(),
                ContentWriteDenial::Content(RelationalContentDenial::Execution(
                    RelationalExecutionDenialCause::Cancelled
                ))
            ));
        }
        other => panic!("original cancellation was lost: {other:?}"),
    }
    assert_eq!(world::pin(&host).selected_commit(), &before);
    assert!(world::observe(&host, "absent-after-cancel").is_none());
    assert!(matches!(
        world::edit(&host, 0, vec![7, -11, 23], vec![], "fresh"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
#[test]
fn content_projection_budget_refuses_the_call_instead_of_returning_optional_absence() {
    let host = world::install();
    world::edit(&host, 0, vec![7], vec![], "initial");
    let before = world::pin(&host).selected_commit().clone();
    let refused = world::try_edit(&host, 15, vec![], vec![], "read-budget").unwrap_err();
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Projection(cause),
    ) = refused
    else {
        panic!("the content read budget must veto projection completion");
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOperationProjectionDenialKind::InvariantAdmission(
            WorthQueryInvariantProjectionDenialKind::ValueDecode(
                WorthQueryValueDecodeDenial::ContentProjectionWorkExceeded
            )
        )
    );
    assert_eq!(world::pin(&host).selected_commit(), &before);
}

#[test]
fn content_unbound_reference_is_refused_by_the_existing_publication_seal() {
    let host = world::install();
    assert!(matches!(
        world::edit(&host, 0, vec![7, -11, 23], vec![], "first"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let identity = world::observe(&host, "observe-first").unwrap().0;
    let before = world::pin(&host).selected_commit().clone();
    let refused = world::edit(&host, 17, vec![], identity.0.to_vec(), "unbound-write");
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = refused
    else {
        panic!("unbound reference must refuse before publication: {refused:?}");
    };
    assert_eq!(
        denial.content_denial(),
        Some(RelationalContentDenial::FormatBindingMissing)
    );
    assert_eq!(world::pin(&host).selected_commit(), &before);
}
