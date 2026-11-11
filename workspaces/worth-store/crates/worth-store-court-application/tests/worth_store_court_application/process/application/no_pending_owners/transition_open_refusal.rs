//! A durable World open retains the actual pre-owner refusal with its custody.
use super::*;

#[test]
fn a_durable_open_retains_the_actual_owner_refusal_before_any_native_effect() {
    use worth_runtime_world::facade::{
        NoEffectCause, RuntimeWorldCancellationSource, WorldDurablePublicationOutcome,
    };
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = open(directory.path(), &execution);
    let original = fixture.bootstrap_root(&execution);
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = fixture.prepare_publication(
        &execution,
        original.product_branch().clone(),
        crate::batches::create(0),
        &cancellation.token(),
    );
    cancellation.cancel();
    let outcome = fixture
        .home
        .world()
        .publication_port()
        .execute_durable_without_signal(execution, prepared, &cancellation.token());
    let WorldDurablePublicationOutcome::ProductUnpublished(retained) = outcome else {
        panic!("the actual durable open must retain its custody")
    };
    assert_eq!(
        retained.before_effect_refusal(),
        Some(NoEffectCause::CancelledBeforeEffect)
    );
    assert_eq!(retained.owner_effect_count(), 0);
    assert_eq!(
        &fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        original.product_branch()
    );
    assert!(fixture
        .home
        .physical_serving_for_test()
        .ordered_journal(
            OrderedJournalName::new(worth_store::physical_runtime::RELATIONAL_CANONICAL_COMMITS)
                .unwrap()
        )
        .unwrap()
        .head()
        .unwrap()
        .is_none());
    assert!(retained
        .next_actions()
        .contains(&worth_runtime_world::facade::ProductUnpublishedNextAction::ReopenRequired));
    // The World open is durable; dropping this observation is not rollback.
    drop(retained);
    drop(original);
    drop(fixture);
}
