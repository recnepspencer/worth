//! Sealed real World subjects share names and bytes on their original grant.
use super::artifact_source::Fixture;
use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionRequestArrayBuilder, SerialMemoryBudget,
    SerialRequest,
};
use worth_runtime_world::facade::*;

fn charge(fixture: &mut Fixture, name: &str) -> (u64, u64) {
    let budget = SerialMemoryBudget::new(Some(1 << 20));
    let policy =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&policy);
    assert!(matches!(
        fixture.offer(&request, name),
        WorldDurableBootstrapOutcome::Refused(WorldDurableBootstrapRefusal::Medium(
            WorldDurableMediumDenial::Unavailable
        ))
    ));
    let original = fixture.recorder.subjects.lock().unwrap().pop().unwrap();
    let charged = budget.charged_memory_bytes();
    let mut frame =
        ExecutionRequestArrayBuilder::allocate(request, original.as_bytes().len()).unwrap();
    frame.extend_from_slice(original.as_bytes()).unwrap();
    let frame = frame.seal().unwrap().into_immutable_bytes();
    let frame_charge = budget.charged_memory_bytes() - charged;
    assert_eq!(original.as_bytes(), frame.bytes());
    let with_frame = budget.charged_memory_bytes();
    let shared = original.clone();
    assert!(shared.same_subject(&original));
    assert_eq!(budget.charged_memory_bytes(), with_frame);
    drop(original);
    assert_eq!(budget.charged_memory_bytes(), with_frame);
    assert_eq!(shared.as_bytes(), frame.bytes());
    drop(shared);
    assert_eq!(budget.charged_memory_bytes(), frame_charge);
    drop(frame);
    assert_eq!(budget.charged_memory_bytes(), 0);
    (charged, frame_charge)
}
#[test]
fn sharing_a_world_artifact_keeps_its_names_and_frame_on_the_original_memory_grant() {
    let setup = crate::test_execution_policy::serial_request();
    let mut fixture = Fixture::new(&ExecutionRequest::serial(&setup));
    // The same genuine components are reused after each proven-no-effect
    // outcome; only the authored branch name and its wire length change.
    let (short, short_frame) = charge(&mut fixture, "n");
    let (long, long_frame) = charge(&mut fixture, &"n".repeat(256));
    assert!(
        long - short >= (long_frame - short_frame) + 255,
        "the frame's existing grant cannot fund separately retained names"
    );
    fixture.finish(&ExecutionRequest::serial(&setup));
}

#[test]
fn a_world_artifact_memory_refusal_precedes_name_and_frame_construction_without_leaking_a_grant() {
    let setup = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&setup);
    let mut fixture = Fixture::journal(&execution);
    let root = match fixture.offer(&execution, &"n".repeat(256)) {
        WorldDurableBootstrapOutcome::Performed(performed) => performed.product_branch().clone(),
        other => panic!("actual stored bootstrap: {other:?}"),
    };
    let cancel = RuntimeWorldCancellationSource::new();
    let opened = fixture
        .world
        .publication_port()
        .open_publication_without_signal(
            &execution,
            root.clone(),
            fixture.recorder.program_source(&root),
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            &cancel.token(),
            None,
        );
    assert!(
        matches!(opened, WorldOpenAttemptOutcome::Opened(_)),
        "{opened:?}"
    );
    drop((opened, root));
    let fixture = fixture.reopen(&execution);
    let page = fixture
        .world
        .inspection_port()
        .recovery_page(None, std::num::NonZeroUsize::MIN)
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    let recovery = fixture.world.recovery_port();
    let custody = recovery
        .inspect_recovered_effects(page.rows()[0].handle())
        .unwrap();
    assert_eq!(
        custody.native_receipt(),
        None,
        "actual durable OpenOnly original"
    );
    let heads = fixture.recorder.heads();
    let budget = SerialMemoryBudget::new(Some(0));
    let policy =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let custody =
        match recovery.release_recovered_effects(&ExecutionRequest::serial(&policy), custody) {
            WorldRecoveredReleaseOutcome::Refused {
                cause:
                    WorldDurableMediumDenial::Request(worth_execution::WorkCeilingDenial::Admission(
                        worth_execution::LeaseDenial::MemoryExhausted(
                            worth_execution::MemoryLimitDenial::Capacity { admitted: 0, .. },
                        ),
                    )),
                custody,
            } => custody,
            other => panic!(
                "the real artifact name allocation refuses before journal publication: {other:?}"
            ),
        };
    assert_eq!(budget.charged_memory_bytes(), 0);
    assert_eq!(fixture.recorder.heads(), heads);
    assert!(recovery
        .released_original_page(&execution, None, std::num::NonZeroUsize::MIN)
        .unwrap()
        .rows()
        .is_empty());
    assert!(matches!(
        recovery.release_recovered_effects(&execution, custody),
        WorldRecoveredReleaseOutcome::Released(_)
    ));
    assert_eq!(fixture.recorder.heads(), [heads[0], heads[1] + 1, heads[2]]);
    assert_eq!(
        recovery
            .released_original_page(&execution, None, std::num::NonZeroUsize::MIN)
            .unwrap()
            .rows()
            .len(),
        1
    );
    drop(page);
    fixture.finish(&execution);
}
