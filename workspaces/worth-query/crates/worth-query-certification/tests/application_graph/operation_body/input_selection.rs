//! Exact captured input selection through ordinary native body execution.
use super::{
    binding::BodyBinding,
    host_limits,
    input::{BodyInputBinding, SetRetentionInput},
    install, invoke, invoke_input, program, read_retention, DocumentRetentionSchema, Outcome,
    DOCUMENT_IDENTITY,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::{
    application_installation::{in_memory_program, WorthQueryInMemoryApplicationDenial},
    declaration::application_operation::{
        ApplicationEncodedInput, ApplicationMutationIdentities, ApplicationOwnedMutationIdentities,
    },
    primary_graph::{WorthQueryOperationBodyDenial, WorthQueryPrimaryGraphInstallationDenialKind},
};

#[test]
fn authored_body_assigns_the_once_encoded_input_not_a_later_serialization() {
    let app = install::<100>();
    let calls = Arc::new(AtomicU64::new(0));
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 42,
        serialization_calls: Some(calls.clone()),
    };
    assert!(matches!(
        invoke_input(&app, 1000, input),
        Outcome::Committed { .. }
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(read_retention(&app), 42);
}

#[test]
fn authored_body_input_replay_and_intent_drift_preserve_the_first_value() {
    let app = install::<100>();
    assert!(matches!(invoke(&app, 1001), Outcome::Committed { .. }));
    assert!(matches!(invoke(&app, 1001), Outcome::AlreadyCommitted(_)));
    let drift = invoke_input(
        &app,
        1001,
        SetRetentionInput {
            identity: DOCUMENT_IDENTITY.to_owned(),
            retention_days: 43,
            ..Default::default()
        },
    );
    assert!(matches!(drift, Outcome::IdempotencyIntentDrift));
    assert_eq!(read_retention(&app), 99);
}

#[test]
fn authored_body_missing_or_wrong_family_input_denies_without_a_write() {
    let wrong = install::<101>();
    assert!(matches!(
        invoke(&wrong, 1002),
        Outcome::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch { .. })
    ));
    assert_eq!(read_retention(&wrong), 7);
    let missing = install::<102>();
    assert!(matches!(
        invoke(&missing, 1003),
        Outcome::DomainDenied(WorthQueryOperationBodyDenial::InputMismatch { .. })
    ));
    assert_eq!(read_retention(&missing), 7);
    assert_ne!(
        wrong.installed_program().revision(),
        missing.installed_program().revision()
    );
}

#[test]
fn authored_body_input_selection_requires_capture_at_installation() {
    let schema = program::schema::<103>();
    let result = in_memory_program(
        program::validated::<103>(&schema),
        schema,
        (program::Installation::Native,),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        |_phase, _graph, _schema| panic!("missing capture must deny before initialization"),
    );
    match result {
        Err(WorthQueryInMemoryApplicationDenial::Contributions(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::OperationBodyUnavailable
        ),
        Err(other) => panic!("expected missing native input capture: {other:?}"),
        Ok(_) => panic!("input body cannot operate on a second serialization"),
    }
}

#[test]
fn authored_body_input_snapshot_survives_preencoded_owned_and_clone_handoffs() {
    let calls = Arc::new(AtomicU64::new(0));
    let input = SetRetentionInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        retention_days: 55,
        serialization_calls: Some(calls.clone()),
    };
    let key = 1004_u64;
    let original =
        ApplicationMutationIdentities::<DocumentRetentionSchema<100>, BodyBinding<100>>::encode(
            &key, &input,
        )
        .unwrap();
    let encoded = original.encoded_input();
    let clone = encoded.clone();
    let rebound = ApplicationMutationIdentities::<DocumentRetentionSchema<100>, BodyBinding<100>>::with_encoded_input(&key, &clone).unwrap();
    assert_eq!(original.input_identity(), rebound.input_identity());
    assert_eq!(
        rebound
            .clone()
            .canonical_input()
            .unwrap()
            .scalar_member("retention_days", ScalarAspectType::UInt64)
            .unwrap()
            .into_value(),
        AspectValue::UInt64(55)
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let owned_calls = Arc::new(AtomicU64::new(0));
    let owned = ApplicationOwnedMutationIdentities::<DocumentRetentionSchema<100>, BodyBinding<100>>::encode_admitted(key, SetRetentionInput { identity: DOCUMENT_IDENTITY.to_owned(), retention_days: 56, serialization_calls: Some(owned_calls.clone()) }, &mut |_| Ok::<(), ()>(())).unwrap();
    assert_eq!(
        owned
            .borrow()
            .encoded_input()
            .canonical_input()
            .unwrap()
            .scalar_member("retention_days", ScalarAspectType::UInt64)
            .unwrap()
            .into_value(),
        AspectValue::UInt64(56)
    );
    assert_eq!(owned_calls.load(Ordering::SeqCst), 1);
    let retained = ApplicationOwnedMutationIdentities::<
        DocumentRetentionSchema<100>,
        BodyBinding<100>,
    >::with_encoded_input(key, &encoded)
    .unwrap();
    drop(encoded);
    drop(clone);
    drop(rebound);
    drop(original);
    assert_eq!(retained.borrow().mutation_input().retention_days, 55);
    assert_eq!(
        retained
            .borrow()
            .canonical_input()
            .unwrap()
            .scalar_member("retention_days", ScalarAspectType::UInt64)
            .unwrap()
            .into_value(),
        AspectValue::UInt64(55)
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "owning the existing captured input must not serialize it again"
    );
    let direct = ApplicationEncodedInput::<BodyInputBinding<100>>::encode(SetRetentionInput {
        retention_days: 57,
        ..Default::default()
    })
    .unwrap();
    assert_eq!(
        direct
            .canonical_input()
            .unwrap()
            .scalar_member("retention_days", ScalarAspectType::UInt64)
            .unwrap()
            .into_value(),
        AspectValue::UInt64(57)
    );
}
