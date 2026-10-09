use std::time::{Duration, Instant};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_relational::facade::transactions::WorkerIntentBatch;
use worth_runtime_world::facade::{NoEffectCause, RuntimeWorldPublicationOutcome};

use super::fixture::{installed_authorization_world, prepare_relational_mutation_on_application};

#[test]
fn cancellation_after_world_preparation_reaches_execution_before_owner_effects() {
    let world = installed_authorization_world(true);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let prepared = prepare_relational_mutation_on_application(
        &world.application,
        WorkerIntentBatch::new("cancel-after-world-preparation"),
        &request,
    );

    cancellation.cancel();
    let outcome = crate::domain_computation::primary_graph::with_test_advancement(|phase| {
        prepared.execute(
            phase
                .request_for_source(0)
                .expect("bounded standalone fixture request"),
        )
    });
    let RuntimeWorldPublicationOutcome::NoEffect(no_effect) = outcome else {
        panic!("cancellation after preparation must prevent owner effects")
    };
    assert_eq!(no_effect.cause(), NoEffectCause::CancelledBeforeEffect);
}
