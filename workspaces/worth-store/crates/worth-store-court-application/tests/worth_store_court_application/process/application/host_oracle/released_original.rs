//! Ordinary Memory release preserves the same keyed business outcome as Home.
use super::*;
use host_observation::{requests, OracleHome};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_host::facade::{
    primary_graph::{
        RuntimeWorldRecoveryDenial, WorthQueryApplicationIdempotencyResolution as Resolution,
        WorthQueryProductUnpublishedRecoveryReleaseDenial as Denial,
        WorthQueryProductUnpublishedRecoveryReleaseFailure as Failure,
    },
    WorthQueryApplicationHomeLocation,
};

#[test]
fn memory_release_preserves_original_key_and_refuses_handler_replay() {
    let request = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let contacts = Arc::new(AtomicUsize::new(0));
    let home = host_observation::opening::open(
        WorthQueryApplicationHomeLocation::MemoryOnly,
        opening::build(),
        Arc::new(AtomicUsize::new(0)),
        contacts.clone(),
        None,
        Default::default(),
        &execution,
    );
    let predecessor = host_observation::projection::observe(home.runtime(), &execution).head;
    let unpublished =
        host_interruption::publication::abandon_after_owner(home.runtime(), 192, 90_991);
    drop(unpublished);
    let runtime = home.runtime().runtime();
    let page = runtime
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap();
    let [row] = page.rows() else {
        panic!("one real Memory original");
    };
    let recovery = runtime
        .readmit_product_publication_recovery(row.handle())
        .unwrap();
    let semantic = serde_json::to_value(
        recovery
            .inspect()
            .unwrap()
            .native_semantic_result()
            .unwrap(),
    )
    .unwrap();
    let count = contacts.load(Ordering::SeqCst);
    let Failure::Recovery(refused) = runtime
        .release_product_publication_recovery(recovery, u64::MAX)
        .unwrap_err()
    else {
        panic!("age refusal must retain original custody");
    };
    assert_eq!(
        refused.denial(),
        Denial::World(RuntimeWorldRecoveryDenial::TooYoung)
    );
    let recovery = refused.into_recovery();
    assert_eq!(
        serde_json::to_value(
            recovery
                .inspect()
                .unwrap()
                .native_semantic_result()
                .unwrap()
        )
        .unwrap(),
        semantic
    );
    runtime
        .release_product_publication_recovery(recovery, 0)
        .unwrap();
    let original = match requests::resolve_original(home.runtime(), 192, 90_991).unwrap() {
        Resolution::ReleasedOriginal(original) => original,
        other => panic!("released Native original cannot become unseen: {other:?}"),
    };
    assert_eq!(
        serde_json::to_value(original.semantic_result()).unwrap(),
        semantic
    );
    assert!(original.original_effect().original_association().is_none());
    assert!(original
        .original_effect()
        .release_journal_position()
        .is_none());
    assert!(original
        .original_effect()
        .release_artifact_digest()
        .is_none());
    assert!(matches!(
        requests::resolve_original(home.runtime(), 192, 90_992).unwrap(),
        Resolution::IntentDrift
    ));
    assert_retry_refused(&home, 192, 90_991);
    assert_eq!(contacts.load(Ordering::SeqCst), count);
    assert_eq!(
        host_observation::projection::observe(home.runtime(), &execution).head,
        predecessor
    );
    assert!(runtime
        .product_publication_recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    drop(page);
    // Directory lifecycle only, as established by bed4f954f5d; a Memory Home
    // cannot prove physical closure through this door (Host README).
    let refused = home.close(&execution).unwrap_err();
    assert!(refused.directory_denial().is_none());
    assert!(refused.query_denial().is_none());
    let home = refused.resume().unwrap();
    let resumed = match requests::resolve_original(home.runtime(), 192, 90_991).unwrap() {
        Resolution::ReleasedOriginal(original) => original,
        other => panic!("Memory lifecycle refusal must retain terminal meaning: {other:?}"),
    };
    assert_eq!(resumed, original);
    assert_retry_refused(&home, 192, 90_991);
    assert_eq!(contacts.load(Ordering::SeqCst), count);
}

pub(super) fn assert_retry_refused(home: &OracleHome, key: u64, value: u64) {
    use worth_query_host::facade::admission::authenticated_principal::*;
    use worth_query_testkit_host::facade::host::{
        application_home as fixture, application_runtime_surface as identity,
    };
    let scope = WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token());
    let principal = identity::inspection_authentication::authenticate(
        home.runtime().runtime().installed_schema(),
        &scope,
    );
    let denied = home
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(fixture::native_mutation::SetAccountBalance {
            account: "account-001".into(),
            balance: value,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            home.runtime(),
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_err();
    assert_eq!(denied.kind(), worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenialKind::OriginalEffectReleased);
}

pub(super) fn terminal_projection(
    home: &OracleHome,
    key: u64,
    value: u64,
    _execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> super::protocol::ReleasedOriginalProjection {
    let original = match requests::resolve_original(home.runtime(), key, value).unwrap() {
        Resolution::ReleasedOriginal(original) => original,
        other => panic!("terminal original must remain observable: {other:?}"),
    };
    assert!(matches!(
        requests::resolve_original(home.runtime(), key, value + 1).unwrap(),
        Resolution::IntentDrift
    ));
    assert_retry_refused(home, key, value);
    let source = original.original_effect();
    super::protocol::ReleasedOriginalProjection {
        identity: source.identity().ordinal(),
        attempt: source.attempt_identity().ordinal(),
        expected: host_observation::projection::HeadProjection::observe(source.expected_product()),
        association: source.original_association().map(|association| {
            super::protocol::ReleasedAssociationProjection {
                owner: worth_foundational::facade::WorldOwnerCoordinate::Home {
                    owner: association.attempt().owner(),
                },
                branch: association.branch().name().to_owned(),
                incarnation: association.incarnation().ordinal(),
                attempt: association.attempt().ordinal(),
            }
        }),
        release_position: source.release_journal_position(),
        release_digest: source.release_artifact_digest(),
        native_position: source.native_position().unwrap().0,
        native_digest: source.native_artifact_digest().unwrap(),
        semantic: serde_json::to_value(original.semantic_result()).unwrap(),
        key: worth_foundational::facade::CanonicalDigestId::new(
            *original.idempotency_key_identity(),
        ),
        intent: original.recorded_intent().to_owned(),
        affinity: original.recorded_affinity().to_owned(),
        outcome: original.outcome_identity().get(),
        emitted: original.emitted_effect_count(),
        settlement_code: original
            .exact_output_settlement()
            .recorded_code()
            .expect("actual encoded original row"),
    }
}

// Allocation quotes size the refusal probe; they are not durable receipt meaning.
pub(super) fn original_copy_scale(
    home: &OracleHome,
    key: u64,
    value: u64,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> u64 {
    let original = match requests::resolve_original(home.runtime(), key, value).unwrap() {
        Resolution::ReleasedOriginal(original) => original,
        other => panic!("terminal original must remain observable: {other:?}"),
    };
    let source = original.original_effect();
    original
        .semantic_result()
        .created_entities()
        .iter()
        .map(|created| {
            source
                .created_entity(execution, created.assigned)
                .unwrap()
                .unwrap()
                .owned_allocation_capacity_bytes()
        })
        .max()
        .expect("the genuine original includes its keyed row")
}
