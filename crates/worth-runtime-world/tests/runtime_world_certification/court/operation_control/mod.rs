mod combined_race;
mod faults;
mod progress;
mod races;
mod settlement;
use super::*;
use std::{sync::mpsc, time::Duration};
use worth_signal::facade::branch::SignalOwnerOperationBoundary as Boundary;
const WAIT: Duration = Duration::from_secs(5);
fn signal_prepared(
    court: &CompositeSupplyChainCourt,
    head: &ProductBranchObservation,
    token: &RuntimeWorldCancellationToken,
) -> PreparedCompositePublicationWithSignal {
    court
        .world
        .publication_port()
        .prepare_with_signal(
            head.clone(),
            CompositePublicationIntent::with_signal(None),
            token,
            None,
        )
        .unwrap()
}
fn clean(court: &CompositeSupplyChainCourt, outcome: RuntimeWorldPublicationOutcome) {
    match outcome {
        RuntimeWorldPublicationOutcome::Performed(done) => drop(done.consume()),
        RuntimeWorldPublicationOutcome::ProductUnpublished(effects) => {
            let handle = effects.recovery_handle();
            drop(effects);
            assert!(court
                .world
                .recovery_port()
                .release_effects(&handle, 0)
                .unwrap()
                .is_empty());
        }
        RuntimeWorldPublicationOutcome::NoEffect(no) => drop(no),
    }
}
