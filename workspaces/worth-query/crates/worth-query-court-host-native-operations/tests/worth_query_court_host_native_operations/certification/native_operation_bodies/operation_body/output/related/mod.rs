pub(crate) use worth_query_testkit_certification_operation_body::facade::certification::native_operation_bodies::operation_body::output::related::*;

#[test]
fn related_output_of_one_target_is_preserved_without_a_dummy_write() {
    let single = install::<37>();
    let scope = request_scope();
    let principal = authenticate_operator(single.runtime().installed_schema(), &scope);
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.into(),
        retention_days: 99,
        ..Default::default()
    };
    let Outcome::Committed { receipt, .. } = single
        .request(&principal, &scope)
        .advanced()
        .mutate(binding::RelatedIntent(input.clone()))
        .without_source()
        .idempotency(&370)
        .execute_in_program(&single, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    else {
        panic!("one target")
    };
    let selected = receipt
        .outputs_of::<RelatedOutputs<37>>()
        .unwrap()
        .entity::<RelatedOutput<37>>()
        .unwrap()
        .entity_id();
    assert_eq!(
        read_retention(&single),
        7,
        "output-only body has no dummy writes"
    );
    let Outcome::AlreadyCommitted(replay) = single
        .request(&principal, &scope)
        .advanced()
        .mutate(binding::RelatedIntent(input.clone()))
        .without_source()
        .idempotency(&370)
        .execute_in_program(&single, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    else {
        panic!("replay")
    };
    assert_eq!(
        replay
            .outputs_of::<RelatedOutputs<37>>()
            .unwrap()
            .entity::<RelatedOutput<37>>()
            .unwrap()
            .entity_id(),
        selected
    );
}

#[test]
fn related_output_requires_one_target_not_an_arbitrary_member_of_two() {
    let ambiguous = install::<38>();
    let scope = request_scope();
    let principal = authenticate_operator(ambiguous.runtime().installed_schema(), &scope);
    let attempt = ambiguous
        .request(&principal, &scope)
        .advanced()
        .mutate(binding::RelatedIntent(SetRetentionInput {
            identity: DOCUMENT_IDENTITY.into(),
            retention_days: 99,
            ..Default::default()
        }))
        .without_source()
        .idempotency(&380)
        .execute_in_program(&ambiguous, ExecutionAllocationPolicy::SystemAllocation);
    let Err(RequestDenial::Handler(MutationHandlerExecutionDenial::Handler(cause))) = attempt
    else {
        panic!("two targets must not produce a receipt")
    };
    let cause = cause.downcast::<Attempt>().unwrap();
    assert_eq!(cause.kind(), Kind::AmbiguousRelation);
    assert_eq!(cause.subject(), "DocumentMembers");
    assert_eq!(read_retention(&ambiguous), 7);
    for identity in ["member", "second-member"] {
        let result = ambiguous
            .request(&principal, &scope)
            .query(audit_query::AuditRead {
                identity: identity.into(),
            })
            .execute()
            .unwrap();
        let row = &result.rows()[0];
        assert_eq!(row.identity, identity);
        assert_eq!((row.retention_days, row.prior_retention_days), (7, 7));
    }
}
