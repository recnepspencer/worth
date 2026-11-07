//! Cancellation after an actual cold Head read preserves retry and its owner.
use super::super::super::selected_source::MemberStop;
use super::*;
use std::sync::atomic::Ordering;

#[test]
fn cold_head_cancelled_after_member_read_refunds_and_retries_original_data() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&setup);
    let (runtime, medium) = initialized(request);
    let created = create_entity_outcome(request, &runtime, "cancelled-cold-head");
    let entity = created.semantic_result().created_entities()[0].assigned;
    release_test_commit_snapshot(&runtime, &created);
    drop(created);
    let image = capture_external(&runtime, &medium, request);
    medium.file_checkpoint(image.checkpoint().clone());
    let head = member(&image, entity.partition_id, false);
    let history = member(&image, entity.partition_id, true);
    let source = SelectedSource::new(medium.clone());
    let recovered = recover(&image, source.clone(), receiver(&medium), request);
    drop((image, runtime));

    let cancellation = CancellationSource::new();
    source.arm(head, MemberStop::Cancel(cancellation.clone()));
    let budget = SerialMemoryBudget::new(None);
    let caller = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    // The original private source held this region's Head and History only.
    // Keep that exact scope; symbol/other region reads are separate demands.
    let region_reads = || {
        source
            .member_reads
            .lock()
            .unwrap()
            .iter()
            .filter(|&&locator| locator == head || locator == history)
            .count()
    };
    let before = region_reads();
    assert!(matches!(
        read_head(&ExecutionRequest::serial(&caller), &recovered, entity),
        Err(RelationalTransactionReadDenial::Request(
            RelationalExecutionDenialCause::Cancelled
        ))
    ));
    assert!(source.fired.load(Ordering::Relaxed));
    assert_eq!(budget.charged_memory_bytes(), 0);
    let stopped = region_reads();
    assert!(
        stopped > before,
        "cancellation follows an actual member read"
    );
    read_head(request, &recovered, entity).unwrap();
    let retried = region_reads();
    assert!(retried > stopped, "the refusal must not remain cached");
    assert!(
        retried - before < 4,
        "a stopped Head retains the original retry bound"
    );
}
