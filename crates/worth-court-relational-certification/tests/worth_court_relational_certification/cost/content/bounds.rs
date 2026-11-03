use super::stimulus::grid;
use crate::content::fixture::Codec;
use crate::content::transaction::{fixture_registry, FORMAT};
use std::sync::Arc;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::ContentFormat;
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::{content::*, mvcc::*, runtime::*};

#[test]
fn content_complete_frame_and_staged_limits_refuse_without_publication_or_leaks() {
    let cancel = CancellationSource::new();
    let budget = SerialMemoryBudget::new(Some(256 * 1024 * 1024));
    let original = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    let execution = ExecutionRequest::serial(&original);
    let (registry, _) = fixture_registry(
        Arc::new(Codec {
            descriptor: ContentFormat::new(super::stimulus::NAME, 1).unwrap(),
            alter: false,
        }),
        SchemaVersionId(1),
    );
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .unwrap();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let before = basis.descriptor().clone();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let mut input = grid(16 * 1024 * 1024);
    input.samples.push(0);
    input.columns += 1;
    assert_eq!(
        transaction
            .admit_content(&execution, FORMAT, &input)
            .unwrap_err(),
        RelationalContentDenial::SizeLimit {
            observed: 16 * 1024 * 1024 + 8,
            maximum: 16 * 1024 * 1024
        }
    );
    assert_eq!(budget.charged_memory_bytes(), 0);
    input.samples.pop();
    input.columns -= 1;
    for i in 0..4 {
        input.samples[0] = i;
        transaction
            .admit_content(&execution, FORMAT, &input)
            .unwrap();
    }
    let held = budget.charged_memory_bytes();
    input.samples[0] = 99;
    assert_eq!(
        transaction
            .admit_content(&execution, FORMAT, &input)
            .unwrap_err(),
        RelationalContentDenial::SizeLimit {
            observed: 80 * 1024 * 1024,
            maximum: 64 * 1024 * 1024
        }
    );
    assert_eq!(budget.charged_memory_bytes(), held);
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        &before
    );
    cancel.cancel();
    drop(transaction);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
