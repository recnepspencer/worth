//! A genuine issued row never authorizes a sibling source stage.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestMutationDenial as Mutation,
    WorthQueryPerformedMutationExecutionDenial as Execution,
};

#[test]
fn issued_sibling_source_is_refused_before_work_or_publication() {
    let _guard = crate::checkpoint_recovery::checkpoint_recovery_test_guard();
    let world = seeded_world::world(FAMILIES[1], SEED);
    let key = world.members[0].key;
    let control = Arc::new(control::Control::new(world, true, false, 0, false));
    let app = installation::install(control.clone());
    let (scope, principal) = installation::authenticate(&app);
    let request = app.request(&principal, &scope);
    let before = request.query(source::Read { key }).execute().unwrap();
    let output = request.query(facts::Read { key }).execute().unwrap();
    let sibling = request.query(source::Read { key: 100 }).execute().unwrap();
    assert_ne!(before.rows()[0].key, sibling.rows()[0].key);
    let mut envelope = request
        .edit_envelope()
        .idempotency(0x6a01_u64)
        .runtime_stages(&app)
        .ordered_authored()
        .unwrap();
    envelope
        .push_stage_with_source::<application::Root, _, _>(
            edit::Input { key, revision: 1 },
            sibling.observed_sources()[0].clone(),
        )
        .unwrap();
    control.state.lock().unwrap().armed = true;
    let (outcome, writes) =
        worth_relational::facade::inspection::observe_maintenance_work(1, || {
            envelope.execute_performed(
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
        });
    let denial = outcome.expect_err("an issued sibling is not the edited source");
    assert!(matches!(
        denial.cause(),
        Execution::Mutation(Mutation::Handler(_))
    ));
    assert!(
        format!("{:?}", denial.cause()).contains("ForeignModel"),
        "the existing source affinity owner must reject the actual root mismatch: {denial:?}"
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert!(control.state.lock().unwrap().entered.is_empty());
    assert!(control.declarations.lock().unwrap().is_empty());
    let after = request.query(source::Read { key }).execute().unwrap();
    assert_eq!(after.rows(), before.rows());
    let after_output = request.query(facts::Read { key }).execute().unwrap();
    assert_eq!(after_output.rows(), output.rows());
}
