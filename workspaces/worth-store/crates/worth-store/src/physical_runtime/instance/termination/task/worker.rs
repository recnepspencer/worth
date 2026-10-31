use super::{
    PhysicalStoreCloseObservation, PhysicalStoreCloseOutcome, PhysicalStoreCloseStartRefused,
};
use crate::physical_runtime::ServingPhysicalRuntime;
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::thread::JoinHandle;

pub(super) enum PlanObserver {
    None,
    #[cfg(feature = "certification-test-authority")]
    Observe(Box<dyn FnOnce(&super::super::PhysicalStoreClosePlan) + Send>),
}

#[expect(
    clippy::disallowed_methods,
    reason = "category: physical-store-termination; purpose: finish one accepted physical Store close independently of caller waiting; source: worth_store::physical_runtime::instance::termination::task::worker::spawn_close_worker; exact lexical counts: std::thread::Builder::new=1, std::thread::Builder::spawn=1 (two gated calls, one worker); at most one accepted worker per consumed ServingPhysicalRuntime"
)]
pub(super) fn spawn_close_worker(
    serving: ServingPhysicalRuntime,
    sender: mpsc::Sender<PhysicalStoreCloseOutcome>,
    observation: Arc<OnceLock<PhysicalStoreCloseObservation>>,
    observer: PlanObserver,
) -> Result<JoinHandle<()>, PhysicalStoreCloseStartRefused> {
    // Failed spawn destroys its closure. Keep the actual serving owner outside
    // that closure until acceptance so refusal cannot drop it in the caller.
    let packet = Arc::new(Mutex::new(Some(serving)));
    let worker_packet = Arc::clone(&packet);
    let spawned = std::thread::Builder::new()
        .name("worth-store-close".into())
        .spawn(move || {
            let serving = worker_packet
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
                .expect("one accepted worker takes the actual Store");
            // Includes the existing blocking scrub Drop before plan construction.
            let plan = serving.close_plan();
            let _installed = observation.set(plan.observation());
            match observer {
                PlanObserver::None => {}
                #[cfg(feature = "certification-test-authority")]
                PlanObserver::Observe(observe) => observe(&plan),
            }
            let actual = plan.execute();
            // Observer abandonment does not interrupt the owned close. Failed
            // delivery destroys only its already-produced outcome in this worker.
            let _delivery = sender.send(actual);
        });
    match spawned {
        Ok(worker) => Ok(worker),
        Err(error) => {
            let serving = packet
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
                .expect("refused spawn never ran the owner-taking closure");
            Err(PhysicalStoreCloseStartRefused::new(serving, error))
        }
    }
}
