//! Actual native guards consume retained input and carry graph facts through publication.
use super::{
    authenticate_operator, binding, input::SetRetentionInput, install, invoke, invoke_input,
    program, read_retention, request_scope, Outcome, DOCUMENT_IDENTITY,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    declaration::application_schema::{
        validate_portable_application_schema_freshly, WorthQueryPortableApplicationSchemaRecord,
    },
    primary_graph::{WorthQueryApplicationUncommitted, WorthQueryOperationBodyDenial},
    runtime::ExecutionAllocationPolicy,
};

fn input(value: u64) -> SetRetentionInput {
    SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: value,
        ..Default::default()
    }
}
pub(super) fn body<const DESTINATION: u64>(
) -> worth_query_host::facade::declaration::application_schema::ApplicationOperationBody<
    super::DocumentRetentionSchema<DESTINATION>,
    super::SetRetention,
    SetRetentionInput,
> {
    use super::{
        DocumentAuditSourceField, DocumentDraftNoteField, DocumentRetentionField,
        DocumentRetentionSchema, SetRetention,
    };
    if DESTINATION == 116 {
        return SetRetention::reference::<DocumentRetentionSchema<DESTINATION>>()
            .body()
            .expect_input(DocumentDraftNoteField::reference(), "identity")
            .unwrap()
            .assign_input(DocumentRetentionField::reference(), "retention_days")
            .unwrap()
            .finish()
            .unwrap();
    }
    if matches!(DESTINATION, 110..=115) {
        let operation = SetRetention::reference::<DocumentRetentionSchema<DESTINATION>>();
        let body = operation.body();
        let body = if matches!(DESTINATION, 113 | 114) {
            body.expect_input(DocumentDraftNoteField::reference(), "identity")
        } else if DESTINATION == 115 {
            body.expect_input(DocumentAuditSourceField::reference(), "retention_days")
        } else {
            body.expect_input(
                DocumentRetentionField::reference(),
                match DESTINATION {
                    111 => "identity",
                    112 => "missing",
                    _ => "retention_days",
                },
            )
        }
        .unwrap();
        return body
            .assign(DocumentRetentionField::reference(), 99)
            .unwrap()
            .finish()
            .unwrap();
    }
    unreachable!("only input-guard fixture modes select this authoring recipe")
}
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
    assert_eq!(
        validate_portable_application_schema_freshly(
            WorthQueryPortableApplicationSchemaRecord::from_untrusted_parts(parts)
        )
        .unwrap_err(),
        ApplicationSchemaDeclarationDenial::MissingOperationDecisionReadDependency
    );
}
