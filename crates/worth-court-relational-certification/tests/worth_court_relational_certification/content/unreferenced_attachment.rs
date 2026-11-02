//! Final-candidate closure, rather than draft timing, decides attachment custody.
use super::{fixture::*, original, transaction::*};
use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy, ExecutionRequest, ExecutionWorkCeiling,
};
use worth_foundational::facade::ContentRefId;
use worth_relational::facade::{
    content::{RelationalContentAttachments, RelationalContentDenial},
    mvcc::RelationalTransactionIntent,
    transactions::*,
};

#[test]
fn unreferenced_attachment_refuses_with_literal_identity_and_releases_custody() {
    let cancel = CancellationSource::new();
    let original = original(&cancel);
    let execution = ExecutionRequest::serial(&original);
    let (runtime, contract) = fixture();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let before = original.memory().unwrap().available_memory_bytes();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .admit_content(&execution, FORMAT, &grid())
        .unwrap();
    let mut used = grid();
    used.samples[0] = 19;
    let identity = transaction
        .admit_content(&execution, FORMAT, &used)
        .unwrap();
    transaction
        .push_batch(
            batch(&contract, identity),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(original.memory().unwrap().available_memory_bytes() < before);
    let TransactionCommitError::Conflict { error, .. } =
        transaction.commit(&execution, &runtime).unwrap_err()
    else {
        panic!("expected an exact attachment refusal");
    };
    // Independently computed from the literal sample-grid frame, not the writer's ID.
    let golden =
        ContentRefId::from_hex("f4fb54508d898711c5770a58e4cff9989c1c302e7969d842891c0f5f02dc78e8")
            .unwrap();
    assert_eq!(
        error.class,
        ConflictClass::ContentAdmission {
            denial: RelationalContentDenial::UnreferencedAttachment { identity: golden },
        }
    );
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    assert_eq!(selected.descriptor(), basis.descriptor());
    assert_eq!(original.memory().unwrap().available_memory_bytes(), before);

    {
        let authored_frame = basis.encode_content(&execution, FORMAT, &grid()).unwrap();
        let needed_frame = basis.encode_content(&execution, FORMAT, &used).unwrap();
        let mut authored = RelationalContentAttachments::default();
        authored.attach(&execution, &authored_frame).unwrap();
        let held = original.memory().unwrap().available_memory_bytes();
        let mut previous_acquisition = None;
        let mut selector_invoice = None;
        for count in [10, 100, 1000] {
            let mut retained = RelationalContentAttachments::default();
            retained.attach(&execution, &needed_frame).unwrap();
            for i in 0..count {
                let mut extra = grid();
                extra.samples[0] = 10_000 + i;
                let frame = basis.encode_content(&execution, FORMAT, &extra).unwrap();
                retained.attach(&execution, &frame).unwrap();
            }
            // Full availability admission is deliberately measured separately.
            let started = std::time::Instant::now();
            let (reads, acquisition) = ExecutionWorkCeiling::new(None)
                .run_with_checkpoint(None, Some(&original), |_| {
                    let mut reads = basis
                        .prepare_content_reads_with_retained(&execution, &authored, &retained)
                        .unwrap();
                    let actual = reads
                        .read(&execution, needed_frame.identity(), FORMAT)
                        .unwrap();
                    assert_eq!(actual.payload(), needed_frame.payload());
                    reads
                })
                .unwrap();
            let acquisition_us = started.elapsed().as_micros();
            let before_selection = original.memory().unwrap().available_memory_bytes();
            let started = std::time::Instant::now();
            let (selected, selection) = ExecutionWorkCeiling::new(None)
                .run_with_checkpoint(None, Some(&original), |_| {
                    reads.attachments_for_reads(&execution, &authored)
                })
                .unwrap();
            let selected = selected.unwrap();
            let selection_us = started.elapsed().as_micros();
            let selected_ids = selected
                .records()
                .map(|record| record.identity())
                .collect::<Vec<_>>();
            let mut transaction = runtime
                .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                .unwrap();
            transaction.attach_content(&execution, &selected).unwrap();
            drop(selected);
            transaction
                .push_batch(
                    batch(&contract, needed_frame.identity()),
                    execution.allocation_policy(),
                )
                .unwrap();
            let TransactionCommitError::Conflict { error, .. } =
                transaction.commit(&execution, &runtime).unwrap_err()
            else {
                panic!("authored surplus must not be pruned by read availability");
            };
            assert_eq!(
                error.class,
                ConflictClass::ContentAdmission {
                    denial: RelationalContentDenial::UnreferencedAttachment { identity: golden },
                }
            );
            assert_eq!(selected_ids, vec![golden, needed_frame.identity()]);
            assert_eq!(
                runtime
                    .observe_branch(&runtime.main_branch_identity())
                    .unwrap()
                    .1
                    .descriptor(),
                basis.descriptor()
            );
            assert_eq!(
                original.memory().unwrap().available_memory_bytes(),
                before_selection
            );
            let acquisition = acquisition.charged_work();
            let selection = selection.charged_work();
            assert!(selection > 0, "selection must spend original work");
            eprintln!("content availability n={count} acquisition={acquisition} selector={selection} acquisition_us={acquisition_us} selector_us={selection_us}");
            if let Some(prior) = previous_acquisition {
                assert!(
                    acquisition > prior * 5 && acquisition < prior * 25,
                    "full availability must be paid with bounded ordered-index growth"
                );
            }
            if let Some(prior) = selector_invoice {
                assert_eq!(
                    selection, prior,
                    "selector must not scan unrelated availability"
                );
            }
            previous_acquisition = Some(acquisition);
            selector_invoice = Some(selection);
        }
        assert_eq!(original.memory().unwrap().available_memory_bytes(), held);
    }
    assert_eq!(original.memory().unwrap().available_memory_bytes(), before);

    // An attachment already referenced by an unchanged record is not surplus.
    let mut first = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let identity = first.admit_content(&execution, FORMAT, &grid()).unwrap();
    first
        .push_batch(
            batch(&contract, identity),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    first.commit(&execution, &runtime).unwrap();
    let (_, selected) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut next = runtime
        .begin_branch_transaction(&selected, RelationalTransactionIntent::ordinary())
        .unwrap();
    next.admit_content(&execution, FORMAT, &grid()).unwrap();
    let identity = next.admit_content(&execution, FORMAT, &used).unwrap();
    let mut second = batch(&contract, identity);
    let MutationIntent::Create(CreateIntent::EntityAspects(intent)) = &mut second.intents[0] else {
        panic!("fixture create");
    };
    intent.client_key = worth_relational::facade::symbols::ClientKey::raw("second-grid");
    next.push_batch(second, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    next.commit(&execution, &runtime).unwrap();
}
