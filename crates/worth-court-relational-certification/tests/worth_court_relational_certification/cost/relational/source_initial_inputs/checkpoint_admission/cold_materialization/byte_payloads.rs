//! Large byte Head/Full denial, retry and last-owner funding observations.
use crate::relational::durable_publication::source_initial_inputs::{
    checkpoint_admission::cold_materialization::{
        byte_fixture::{assert_bytes, ByteHome},
        member, read_head, update,
    },
    selected_source::MemberStop,
};
use std::sync::{atomic::Ordering, Arc, Mutex};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    mvcc::RelationalTransactionReadDenial, transactions::RelationalExecutionDenialCause,
};
use worth_testkit_relational::{
    aspect_storage::publications::{begin_main_transaction, release_test_commit_snapshot},
    durable_publication::create_entity_outcome,
};
const PAYLOAD: usize = 64 * 1024;
const ROOM: u64 = 2 * PAYLOAD as u64;

#[test]
fn cold_inline_bytes_head_refuses_after_member_read_then_retains_exact_payload_through_edit() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&setup);
    for count in [8, PAYLOAD] {
        let home = ByteHome::new(request, count);
        let (runtime, source) = home.reopen(request);
        let budget = SerialMemoryBudget::new(Some(64 << 20));
        let caller =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        let limited = &ExecutionRequest::serial(&caller);
        let held = Arc::new(Mutex::new(None));
        source.arm(
            member(&home.image, home.entity.partition_id, false),
            MemberStop::MemoryRoom {
                held: held.clone(),
                budget: budget.clone(),
                room: ROOM,
            },
        );
        let result = read_head(limited, &runtime, home.entity);
        assert!(
            source.fired.load(Ordering::Relaxed),
            "pressure followed the actual Head read"
        );
        if count == 8 {
            result.unwrap();
        } else {
            assert!(
                matches!(
                    result,
                    Err(RelationalTransactionReadDenial::Request(
                        RelationalExecutionDenialCause::PolicyMemoryExhausted { .. }
                    ))
                ),
                "{result:?}"
            );
        }
        drop(held.lock().unwrap().take().unwrap());
        if count == PAYLOAD {
            assert_eq!(budget.charged_memory_bytes(), 0);
        }
        let mut transaction = begin_main_transaction(&runtime);
        let record = transaction.read_entity(limited, home.entity).unwrap();
        assert_bytes(record.base().unwrap(), count, 0x51);
        drop((record, transaction));
        if count == PAYLOAD {
            assert!(budget.charged_memory_bytes() >= 2 * PAYLOAD as u64);
        }
        drop(caller);
        // Keep the captured region to its single byte record. The edit target
        // enters only after the controlled read, under the ordinary setup request.
        let editable = create_entity_outcome(request, &runtime, "editable-after-read");
        let editable_id = editable.semantic_result().created_entities()[0].assigned;
        release_test_commit_snapshot(&runtime, &editable);
        drop(editable);
        update(request, &runtime, editable_id);
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        drop(runtime);
        if count == PAYLOAD {
            assert!(budget.charged_memory_bytes() >= 2 * PAYLOAD as u64);
        }
        drop(basis);
        assert_eq!(budget.charged_memory_bytes(), 0);
    }
}

#[test]
fn cold_inline_bytes_history_refuses_after_read_then_retries_full_with_exact_old_values() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&setup);
    for count in [8, PAYLOAD] {
        let home = ByteHome::new(request, count);
        let (runtime, source) = home.reopen(request);
        let budget = SerialMemoryBudget::new(Some(64 << 20));
        let caller =
            SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
        let limited = &ExecutionRequest::serial(&caller);
        let held = Arc::new(Mutex::new(None));
        source.arm(
            member(&home.image, home.entity.partition_id, true),
            MemberStop::MemoryRoom {
                held: held.clone(),
                budget: budget.clone(),
                room: ROOM,
            },
        );
        let result = runtime.storage_access().outgoing_relations_for_entity(
            limited,
            home.entity,
            home.current_version,
        );
        assert!(
            source.fired.load(Ordering::Relaxed),
            "pressure followed the actual History read"
        );
        if count == 8 {
            assert!(result.unwrap().is_empty());
        } else {
            assert!(
                matches!(
                    result,
                    Err(RelationalExecutionDenialCause::PolicyMemoryExhausted { .. })
                ),
                "{result:?}"
            );
            let before =
                budget.limit().unwrap() - held.lock().unwrap().as_ref().unwrap().bytes() - ROOM;
            drop(held.lock().unwrap().take().unwrap());
            assert!(
                budget.charged_memory_bytes() <= before,
                "failed Full added no retained scratch"
            );
        }
        if count == 8 {
            drop(held.lock().unwrap().take().unwrap());
        }
        let retry_budget = SerialMemoryBudget::new(None);
        let retry = SerialRequest::from_memory(
            retry_budget.clone(),
            CancellationSource::new().token(),
            None,
        );
        assert!(runtime
            .storage_access()
            .outgoing_relations_for_entity(
                &ExecutionRequest::serial(&retry),
                home.entity,
                home.current_version
            )
            .unwrap()
            .is_empty());
        if count == PAYLOAD {
            assert!(
                retry_budget.charged_memory_bytes() >= 2 * PAYLOAD as u64,
                "actual Full history backing retains the retry request"
            );
        }
        let view = runtime.inspect_what_happened().open_historical_view(request, home.old_version,
            worth_relational::facade::inspection::HistoricalInspectionMode::AllowCanonicalReconstruction)
            .unwrap().view.unwrap();
        assert_bytes(view.read_view.get_entity(home.entity).unwrap(), count, 0x17);
        drop((view, runtime, caller, retry));
        assert_eq!(budget.charged_memory_bytes(), 0);
        assert_eq!(retry_budget.charged_memory_bytes(), 0);
    }
}
