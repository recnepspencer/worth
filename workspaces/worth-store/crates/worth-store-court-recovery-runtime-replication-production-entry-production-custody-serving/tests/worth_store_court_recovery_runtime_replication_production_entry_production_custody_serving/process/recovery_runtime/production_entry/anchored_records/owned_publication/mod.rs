//! Real physical steps, not a Native checkpoint/acknowledgement substitute.
use worth_execution::{CancellationSource, SerialMemoryBudget};
use worth_store::physical_runtime::production::PhysicalMutationCheckpoint;
use worth_store::physical_runtime::*;
mod checkpoint;
mod duplicate_refusal;
mod fixture;
mod interruption;
mod owner_loss;
use fixture::*;

#[test]
fn multi_batch_publication_resumes_the_same_member_and_preserves_late_anchor_completion() {
    let world = world("owned-multi-batch");
    let records = world.serving().anchored_records().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let port = Port::new(&memory, 3, Delivery::Normal);
    let (operation, recovery) = start(&world, 3, &memory, &cancellation, None);
    let identity = operation.observe().identity();
    let gate = pause(
        world.serving(),
        PhysicalMutationCheckpoint::BeforeEffectCutover,
    );
    let operation = pending(operation, &records, &port);
    assert!(gate.0.await_arrival());
    let first = operation.observe().accepted_mutation().unwrap();
    assert_eq!(
        port.registration(0),
        (first, AnchoredRecordStepKind::Members { batch_index: 0 }, 0)
    );
    let operation = pending(operation, &records, &port);
    assert_eq!(operation.observe().accepted_mutation(), Some(first));
    assert!(operation.observe().completed_prefix().is_empty());
    assert!(records.read(name(), 1024).unwrap().is_none());
    drop(operation);
    let mut operation = match recovery.resume() {
        AnchoredRecordPublicationResume::Ready(same) => same,
        _ => panic!("returned live custody must reclaim the same endpoint"),
    };
    assert_eq!(operation.observe().identity(), identity);
    assert_eq!(operation.observe().accepted_mutation(), Some(first));
    gate.0.release();
    for prefix in 1..=3 {
        operation = match next(operation, &records, &port) {
            AnchoredRecordPublicationStep::Ready(same) => same,
            AnchoredRecordPublicationStep::Failed(f) => panic!("member refused: {:?}", f.cause()),
            _ => panic!("member completion must return caller custody"),
        };
        assert_eq!(operation.observe().completed_prefix().len(), prefix);
        assert!(records.read(name(), 1024).unwrap().is_none());
    }
    let prefix = operation.observe().completed_prefix().to_vec();
    let gate = pause(
        world.serving(),
        PhysicalMutationCheckpoint::AfterRootReplacement,
    );
    let operation = pending(operation, &records, &port);
    assert!(gate.0.await_arrival());
    let (anchor, kind, batches) = port.registration(3);
    assert_eq!(kind, AnchoredRecordStepKind::Anchor);
    assert_eq!(batches, 3);
    cancellation.cancel();
    let stopped = operation.stop();
    gate.0.release();
    let actual = match settle(stopped, &records) {
        AnchoredRecordStoppedPoll::Settled(actual) => actual,
        AnchoredRecordStoppedPoll::Failed(f) => {
            panic!("effectful anchor lost completion: {:?}", f.cause())
        }
        _ => unreachable!(),
    };
    let receipt = actual.terminal().unwrap();
    let physical = actual.step_terminal().unwrap();
    assert_eq!(physical.mutation_identity(), anchor);
    let PhysicalMutationOutcome::Completed(done) = physical.outcome() else {
        panic!("post-cutover actual fate lost")
    };
    assert_eq!(
        receipt.version().record().allocation_epoch(),
        done.persisted_records()[0].allocation_epoch()
    );
    assert_eq!(
        receipt.version().record().ordinal(),
        done.persisted_records()[0].ordinal()
    );
    assert_eq!(receipt.members(), prefix);
    let current = records.read(name(), 1024).unwrap().unwrap();
    assert_eq!(current.version(), receipt.version());
    assert_eq!(current.bytes(), PAYLOAD);
    for (index, locator) in current.members().iter().enumerate() {
        assert_eq!(
            records
                .read_member(name(), *locator, MEMBER_BYTES as u64)
                .unwrap(),
            vec![index as u8 + 1; MEMBER_BYTES]
        );
    }
    drop(port);
    assert_eq!(
        memory.charged_memory_bytes(),
        0,
        "terminal descriptions must be inert"
    );
}
