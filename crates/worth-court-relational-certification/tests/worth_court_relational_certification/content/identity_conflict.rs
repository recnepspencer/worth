//! A real staged duplicate must not publish when its exact witness disagrees.
use super::{fixture::*, original, transaction::*};
use std::{panic::AssertUnwindSafe, sync::Arc};
use worth_execution::{CancellationSource, ExecutionRequest};
use worth_foundational::facade::ContentFormat;
use worth_relational::facade::{
    content::RelationalContentDenial,
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntimeApi,
    schema::SchemaVersionId,
    transactions::{ConflictClass, TransactionCommitError},
};

#[test]
fn duplicate_content_disagreement_refuses_real_admission_with_literal_identity_conflict() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let (registry, contract) = fixture_registry(
        Arc::new(Codec {
            descriptor: ContentFormat::new("sample-grid", 1).unwrap(),
            alter: false,
        }),
        SchemaVersionId(1),
    );
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry.clone())
        .build()
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let before = original.memory().unwrap().available_memory_bytes();
    let transaction = || {
        let mut tx = runtime
            .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        let first = tx.admit_content(&execution, FORMAT, &grid()).unwrap();
        let second = tx.admit_content(&execution, FORMAT, &grid()).unwrap();
        assert_eq!(first, second);
        tx.push_batch(batch(&contract, first), execution.allocation_policy())
            .unwrap();
        tx
    };
    let tx = transaction();
    let refusal = registry
        .with_injected_duplicate_content_corruption_for_test(FORMAT, || {
            tx.commit(&execution, &runtime)
        })
        .unwrap()
        .unwrap_err();
    let TransactionCommitError::Conflict { error, .. } = refusal else {
        panic!("expected the exact Native admission conflict");
    };
    assert_eq!(
        error.class,
        ConflictClass::ContentAdmission {
            denial: RelationalContentDenial::ContentIdentityConflict,
        }
    );
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
    assert_eq!(original.memory().unwrap().available_memory_bytes(), before);

    // An unwound scope must not poison the next actual duplicate admission.
    assert!(std::panic::catch_unwind(AssertUnwindSafe(|| {
        let _ = registry.with_injected_duplicate_content_corruption_for_test(FORMAT, || {
            panic!("scope cleanup")
        });
    }))
    .is_err());
    transaction().commit(&execution, &runtime).unwrap();
}
