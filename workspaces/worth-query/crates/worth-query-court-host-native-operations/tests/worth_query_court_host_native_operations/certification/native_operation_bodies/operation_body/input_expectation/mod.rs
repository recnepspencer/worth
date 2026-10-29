pub(crate) use worth_query_testkit_certification_operation_body::facade::certification::native_operation_bodies::operation_body::input_expectation::*;

#[test]
fn guard_input_does_not_shift_a_later_assignment_slot() {
    let app = install::<116>();
    assert!(matches!(
        invoke_input(&app, 1160, input(42)),
        Outcome::Committed { .. }
    ));
    assert_eq!(read_retention(&app), 42);
}

#[test]
fn input_guard_uses_original_capture_and_replay_does_not_recheck_changed_source() {
    let app = install::<110>();
    let calls = Arc::new(AtomicU64::new(0));
    let outcome = invoke_input(
        &app,
        1100,
        SetRetentionInput {
            serialization_calls: Some(calls.clone()),
            ..input(7)
        },
    );
    let original = match outcome {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("guarded commit: {other:?}"),
    };
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(read_retention(&app), 99);
    match invoke_input(&app, 1100, input(7)) {
        Outcome::AlreadyCommitted(replay) => {
            assert_eq!(replay.outcome_identity(), original.outcome_identity())
        }
        other => panic!("exact replay: {other:?}"),
    }
    assert!(
        matches!(invoke_input(&app,1101,input(7)),Outcome::DomainDenied(WorthQueryOperationBodyDenial::SourceMismatch{field,..}) if field=="DocumentRetentionField")
    );
    assert_eq!(read_retention(&app), 99);
}

#[test]
fn input_guard_mismatch_and_missing_or_wrong_family_members_do_not_write() {
    let mismatch = install::<110>();
    assert!(
        matches!(invoke_input(&mismatch,1102,input(8)),Outcome::DomainDenied(WorthQueryOperationBodyDenial::SourceMismatch{field,..}) if field=="DocumentRetentionField")
    );
    assert_eq!(read_retention(&mismatch), 7);
    let wrong = install::<111>();
    assert!(
        matches!(invoke(&wrong,1110),Outcome::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch{member}) if member=="identity")
    );
    assert_eq!(read_retention(&wrong), 7);
    let missing = install::<112>();
    assert!(
        matches!(invoke(&missing,1120),Outcome::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch{member}) if member=="missing")
    );
    assert_eq!(read_retention(&missing), 7);
}

#[test]
fn optional_field_guard_requires_a_present_equal_value() {
    let present = install::<113>();
    assert!(matches!(invoke(&present, 1130), Outcome::Committed { .. }));
    assert_eq!(read_retention(&present), 99);
    let absent = install::<114>();
    assert!(
        matches!(invoke(&absent,1140),Outcome::DomainDenied(WorthQueryOperationBodyDenial::SourceMismatch{field,..}) if field=="DocumentDraftNoteField")
    );
    assert_eq!(read_retention(&absent), 7);
}

#[test]
fn guarded_read_only_fact_movement_stales_prepared_publication() {
    let app = install::<115>();
    let scope = request_scope();
    let principal = authenticate_operator(app.runtime().installed_schema(), &scope);
    let key = 1150;
    let mut request = app
        .request(&principal, &scope)
        .advanced()
        .mutate(binding::BodyIntent(input(7)))
        .without_source()
        .idempotency(&key);
    let candidate = match request
        .prepare_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("fresh guard unexpectedly settled"),
    };
    // A different operation changes ONLY the guarded field, not the retention write target.
    assert!(matches!(
        app.request(&principal, &scope)
            .advanced()
            .mutate(binding::SourceIntent(input(8)))
            .without_source()
            .idempotency(&1151)
            .execute_in_program(&app, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap(),
        Outcome::Committed { .. }
    ));
    assert!(matches!(
        candidate.commit(ExecutionAllocationPolicy::SystemAllocation),
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(_))
    ));
    assert_eq!(read_retention(&app), 7);
}

#[test]
fn input_guard_readmits_as_authored_meaning_not_a_host_callback() {
    let schema = program::schema::<110>();
    let restored = validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::project(schema.erased()),
    )
    .unwrap();
    assert_eq!(&restored, schema.erased());
    let changed = program::schema::<111>();
    assert_ne!(schema.identity(), changed.identity());
    // Removing only the guard's read must fail cold closure, not merely runtime execution.
    use worth_query_host::facade::declaration::application_schema::{
        ApplicationOperationDecisionReadTarget, ApplicationSchemaDeclarationDenial,
        ApplicationSchemaMember,
    };
    let guarded = program::schema::<113>();
    let mut parts =
        WorthQueryPortableApplicationSchemaRecord::project(guarded.erased()).into_parts();
    let mut changed = false;
    for member in &mut parts.members {
        if let ApplicationSchemaMember::OperationDecisionRead { operation, target } = member
            && operation == "SetRetention"
            && matches!(target,ApplicationOperationDecisionReadTarget::Field{field,..} if field=="DocumentDraftNoteField")
        {
            *target = ApplicationOperationDecisionReadTarget::Entity {
                entity: "Document".to_owned(),
            };
            changed = true;
        }
    }
    assert!(changed, "the guard must declare its exact read");
    parts.members.sort();
    let denied = validate_portable_application_schema_freshly(
        WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts),
    )
    .unwrap_err();
    assert!(
        matches!(
            &denied,
            ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency {
                operation,
                ..
            } if operation == "SetRetention"
        ),
        "{denied:?}"
    );
}
