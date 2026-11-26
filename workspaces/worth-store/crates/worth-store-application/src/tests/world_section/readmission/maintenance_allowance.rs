//! Issue #423: each automatic run and close capture owns its own allowance.
use super::*;

/// One run here charges roughly 130K-650K work, growing with the Home; the
/// sixteen captures below exceed this allowance many times over together.
const MAINTENANCE_WORK: u64 = 1 << 20;
const CYCLES: u64 = 15;

#[test]
fn repeated_automatic_maintenance_never_exhausts_a_lifetime_allowance_before_close() {
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
    let world = super::readmit_attached_world(&home, builder, bridge, &execution).unwrap();
    let checkpoint = signal.checkpoint_port();
    let home =
        super::finish_attachment(home, world, None, Some(signal), checkpoint).consume(|home| home);
    // Every commit crosses the count threshold, so every run captures.
    let limits = crate::ApplicationHomeCheckpointLimits::new(
        std::num::NonZeroU64::MIN,
        std::num::NonZeroU64::MAX,
    )
    .with_maintenance_allowance(
        std::num::NonZeroU64::new(MAINTENANCE_WORK).unwrap(),
        std::num::NonZeroU64::new(16 << 20).unwrap(),
    );
    let operation = home.checkpoint_operation(limits).unwrap();
    operation.install_home_checkpoint_maintenance().unwrap();
    for cycle in 1..=CYCLES {
        crate::tests::native_schema::commit(
            home.relational().runtime(),
            &execution,
            &format!("maintenance-{cycle}"),
        );
        let automatic = operation
            .automatic_checkpoint_maintenance()
            .unwrap_or_else(|cause| panic!("automatic run {cycle}: {cause:?}"))
            .expect("the crossing is due");
        assert_eq!(automatic.generation(), cycle);
    }
    // Close capture uses the SAME installed worker after every earlier run.
    crate::tests::native_schema::commit(home.relational().runtime(), &execution, "closing");
    let close = operation.checkpoint_for_close().unwrap();
    assert_eq!(close.generation(), CYCLES + 1);
    assert!(!operation.checkpoint_maintenance_is_active());
    drop(operation);
    drop(home);
}
