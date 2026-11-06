//! The recorded-history corruption arm is scoped to its actual receiver and unwind.
use super::*;

#[test]
fn recorded_header_corruption_restores_history_when_the_court_unwinds() {
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    );
    let execution = &ExecutionRequest::serial(&request);
    let world = journey::run(execution);
    let runtime = &world.runtime;
    let basis = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap()
        .1;
    let publication = world
        .retired
        .required_output_publications(&world.installed, execution)
        .next()
        .unwrap()
        .unwrap();
    let admit = || {
        runtime.readmit_authored_output_retirement_successor(
            execution,
            &world.installed,
            &basis,
            publication.meaning(),
        )
    };
    assert!(admit().unwrap().is_some());
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime
            .history_authority()
            .with_corrupt_commit_envelope_for_test(
                world.retired.commit.commit_id,
                Corruption::CascadeIncompleteOriginMembership,
                || {
                    assert!(admit().is_err(), "corruption is armed on this receiver");
                    panic!("deliberate court unwind");
                },
            );
    }));
    let panic = unwind.expect_err("court deliberately unwinds");
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"deliberate court unwind")
    );
    assert!(admit().expect("genuine history survives unwind").is_some());
    assert_eq!(
        runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}
