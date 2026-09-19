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
    let RuntimeWorldPublicationOutcome::NoEffect(no_effect) = prepared.execute() else {
        panic!("cancellation after preparation must prevent owner effects")
    };
    assert_eq!(no_effect.cause(), NoEffectCause::CancelledBeforeEffect);
}
