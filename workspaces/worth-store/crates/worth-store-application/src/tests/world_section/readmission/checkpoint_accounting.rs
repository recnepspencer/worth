//! Manual and automatic accounting share the actual Complete checkpoint original.
use super::*;
#[test]
fn composed_home_manual_and_automatic_checkpoint_share_actual_complete_attempt() {
    let directory = tempfile::tempdir().unwrap();
    let policy = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(16 << 20)),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = ExecutionRequest::serial(&policy);
    let binding = worth_query_testkit_installation::facade::installation::application_home_binding::admitted_home_binding();
    let build = worth_foundational::facade::ApplicationBuildIdentity::parse(
        "11111111-1111-4111-8111-111111111111",
    )
    .unwrap();
    let opening = crate::ApplicationHomeRequest::new(
        directory.path().to_owned(),
        crate::tests::native_schema::definition(),
        binding,
        build,
    );
    let home = execution
        .in_scope(|_| {
            let recovered = crate::recover_home(opening, &execution).unwrap();
            let admitted = crate::admit_home(recovered, &execution).unwrap();
            crate::readmit_relational(admitted, &execution)
                .unwrap()
                .consume(|home| home)
        })
        .unwrap();
    let (builder, bridge, signal, _attachment) = recipe(&home, &execution);
    let medium = home.context.world_slot.as_ref().unwrap().medium();
    let world = super::readmit_attached_world(&home, builder, bridge, &execution).unwrap();
    let checkpoint = signal.checkpoint_port();
    let home =
        super::finish_attachment(home, world, None, Some(signal), checkpoint).consume(|home| home);
    let empty = home
        .checkpoint(&execution)
        .expect("configured C0 complete capture");
    assert_eq!(empty.covered().0, 0);
    crate::tests::native_schema::commit(
        home.relational().runtime(),
        &execution,
        "maintenance-original",
    );
    let first = home
        .checkpoint(&execution)
        .expect("genuine complete Native and World capture");
    assert!(first.covered().0 > empty.covered().0);
    assert_eq!(first.generation(), empty.generation() + 1);
    let limits = crate::ApplicationHomeCheckpointLimits::new(
        std::num::NonZeroU64::MIN,
        std::num::NonZeroU64::new(64 << 20).unwrap(),
    );
    let operation = home.checkpoint_operation(limits).unwrap();
    assert!(operation.after_advancement(&execution).unwrap().is_none());
    let original = home.inspect_checkpoint(limits, &execution).unwrap();
    {
        let fork = home.relational().runtime().fork_port();
        let (_, basis) = fork
            .observe_fork_source(
                home.relational()
                    .runtime()
                    .main_branch_identity()
                    .branch_id(),
            )
            .unwrap();
        fork.fork_branch(
            &execution,
            worth_relational::facade::history::BranchId("automatic-checkpoint".into()),
            basis,
        )
        .unwrap();
    }
    let due = home.inspect_checkpoint(limits, &execution).unwrap();
    assert!(due.due);
    assert_eq!(due.journals[1].head, original.journals[1].head + 1);
    let automatic = operation.after_advancement(&execution).unwrap().unwrap();
    assert_eq!(automatic.generation(), first.generation() + 1);
    assert_eq!(automatic.covered_transitions().0, due.journals[1].head);
    assert!(operation.after_advancement(&execution).unwrap().is_none());
    let byte_limits = crate::ApplicationHomeCheckpointLimits::new(
        std::num::NonZeroU64::new(4096).unwrap(),
        std::num::NonZeroU64::MIN,
    );
    let byte_operation = home.checkpoint_operation(byte_limits).unwrap();
    {
        let fork = home.relational().runtime().fork_port();
        let (_, basis) = fork
            .observe_fork_source(
                home.relational()
                    .runtime()
                    .main_branch_identity()
                    .branch_id(),
            )
            .unwrap();
        fork.fork_branch(
            &execution,
            worth_relational::facade::history::BranchId("automatic-checkpoint-bytes".into()),
            basis,
        )
        .unwrap();
    }
    let byte_due = home.inspect_checkpoint(byte_limits, &execution).unwrap();
    assert!(byte_due.due);
    assert_eq!(byte_due.journals[1].head - byte_due.journals[1].covered, 1);
    assert!(byte_due.journals[1].uncovered_payload_bytes.unwrap() > 0);
    let automatic_bytes = byte_operation
        .after_advancement(&execution)
        .unwrap()
        .unwrap();
    assert_eq!(automatic_bytes.generation(), automatic.generation() + 1);
    assert_eq!(
        automatic_bytes.covered_transitions().0,
        byte_due.journals[1].head
    );
    assert!(operation.after_advancement(&execution).unwrap().is_none());
    assert!(byte_operation
        .after_advancement(&execution)
        .unwrap()
        .is_none());
    let manual = home.checkpoint(&execution).unwrap();
    assert_eq!(manual.generation(), automatic_bytes.generation() + 1);
    assert!(matches!(
        operation.recover_checkpoint(&execution),
        Err(crate::ApplicationHomeCheckpointDenial::NoPendingCheckpoint)
    ));
    drop((operation, byte_operation));
    // The current preparation/selection slice does not certify provider graph
    // retirement or physical Closed. Dispose these real fixture owners explicitly.
    drop((home, medium));
}
