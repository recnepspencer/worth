use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::JoinHandle,
};

use worth_signal::facade::RuntimeClockBasis;

use crate::physical_runtime::work::{
    physical_work_abandonment_channel, PhysicalSignalAspectBindingDigest,
    PhysicalSignalAspectBindingSet, PhysicalWorkAbandonmentInbox, PhysicalWorkAbandonmentPublisher,
};

use super::{
    graph::{PhysicalSignalAbandonmentFailure, PhysicalSignalGraph},
    route::{
        PhysicalSignalRouteCommand, PhysicalSignalRouteMailbox, PhysicalSignalRouteOwner,
        ROUTE_COMMAND_CAPACITY,
    },
    wake::PhysicalSignalWorkerWake,
    PhysicalSignalAdmissionStatus, PhysicalSignalConstructionFailure,
};

pub(super) struct PhysicalSignalGraphWorker {
    routes: Box<[PhysicalSignalRouteOwner]>,
    worker: Option<JoinHandle<()>>,
    shared: PhysicalSignalWorkerShared,
    abandonment: PhysicalWorkAbandonmentPublisher,
}

#[derive(Clone)]
struct PhysicalSignalWorkerShared {
    stopping: Arc<AtomicBool>,
    wake: Arc<PhysicalSignalWorkerWake>,
    clock: Arc<Mutex<RuntimeClockBasis>>,
    admission: PhysicalSignalAdmissionStatus,
    #[cfg(feature = "certification-test-authority")]
    pause_after_dequeue: Arc<Mutex<Option<super::CertificationPhysicalSignalPauseGate>>>,
    #[cfg(feature = "certification-test-authority")]
    fail_next_abandonment: Arc<AtomicBool>,
}

struct PhysicalSignalWorkerBoot {
    bindings: Arc<PhysicalSignalAspectBindingSet>,
    mailboxes: Box<[Arc<PhysicalSignalRouteMailbox>]>,
    shared: PhysicalSignalWorkerShared,
    abandonments: PhysicalWorkAbandonmentInbox,
    ready: mpsc::SyncSender<Result<(), PhysicalSignalConstructionFailure>>,
}

struct PhysicalSignalWorkerLivenessGuard {
    admission: PhysicalSignalAdmissionStatus,
    mailboxes: Box<[Arc<PhysicalSignalRouteMailbox>]>,
    abandonments: PhysicalWorkAbandonmentInbox,
}

impl PhysicalSignalGraphWorker {
    pub(super) fn spawn(
        bindings: Arc<PhysicalSignalAspectBindingSet>,
        admission: PhysicalSignalAdmissionStatus,
    ) -> Result<Self, PhysicalSignalConstructionFailure> {
        let route_ids = bindings
            .bindings()
            .iter()
            .map(|binding| binding.digest())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let shared = PhysicalSignalWorkerShared::new(admission);
        let mailboxes = route_mailboxes(route_ids.len(), &shared);
        let wake: Arc<dyn crate::physical_runtime::work::PhysicalWorkAbandonmentWake> =
            Arc::clone(&shared.wake) as Arc<_>;
        let (abandonment, abandonments) =
            physical_work_abandonment_channel(bindings.capacity().commands(), wake);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(0);
        let worker = spawn_graph_worker(PhysicalSignalWorkerBoot {
            bindings,
            mailboxes: mailboxes.clone(),
            shared: shared.clone(),
            abandonments,
            ready: ready_sender,
        })?;
        let worker = await_graph_readiness(&ready_receiver, worker)?;
        Ok(Self {
            routes: bind_routes(&route_ids, &mailboxes),
            worker: Some(worker),
            shared,
            abandonment,
        })
    }

    pub(super) fn abandonment_publisher(&self) -> PhysicalWorkAbandonmentPublisher {
        self.abandonment.clone()
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn pause_after_dequeue_for_certification(
        &self,
    ) -> super::CertificationPhysicalSignalPauseGate {
        let gate = super::CertificationPhysicalSignalPauseGate::new();
        *self
            .shared
            .pause_after_dequeue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(gate.worker_handle());
        gate
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn fail_next_abandonment_for_certification(&self) {
        self.shared
            .fail_next_abandonment
            .store(true, Ordering::Release);
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn route_depth_for_certification(
        &self,
        route: PhysicalSignalAspectBindingDigest,
    ) -> Option<usize> {
        self.route(route).map(|owner| owner.mailbox.len())
    }

    pub(super) fn len(&self) -> usize {
        self.routes.len()
    }

    pub(super) fn route(
        &self,
        route: PhysicalSignalAspectBindingDigest,
    ) -> Option<&PhysicalSignalRouteOwner> {
        self.routes.iter().find(|owner| owner.route == route)
    }

    pub(super) fn clock_basis(&self) -> Option<RuntimeClockBasis> {
        self.shared.admission.is_available().then(|| {
            *self
                .shared
                .clock
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
        })
    }

    pub(super) fn stop(&mut self) {
        self.shared.admission.revoke();
        self.shared.stopping.store(true, Ordering::Release);
        self.shared.wake.signal();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for PhysicalSignalGraphWorker {
    fn drop(&mut self) {
        self.stop();
    }
}

impl PhysicalSignalWorkerShared {
    fn new(admission: PhysicalSignalAdmissionStatus) -> Self {
        Self {
            stopping: Arc::new(AtomicBool::new(false)),
            wake: Arc::new(PhysicalSignalWorkerWake::new()),
            clock: Arc::new(Mutex::new(RuntimeClockBasis::default())),
            admission,
            #[cfg(feature = "certification-test-authority")]
            pause_after_dequeue: Arc::new(Mutex::new(None)),
            #[cfg(feature = "certification-test-authority")]
            fail_next_abandonment: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Drop for PhysicalSignalWorkerLivenessGuard {
    fn drop(&mut self) {
        self.admission.revoke();
        for mailbox in &self.mailboxes {
            mailbox.clear();
        }
    }
}

fn route_mailboxes(
    route_count: usize,
    shared: &PhysicalSignalWorkerShared,
) -> Box<[Arc<PhysicalSignalRouteMailbox>]> {
    (0..route_count)
        .map(|_| {
            Arc::new(PhysicalSignalRouteMailbox::new(
                Arc::clone(&shared.wake),
                shared.admission.clone(),
                ROUTE_COMMAND_CAPACITY,
            ))
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn bind_routes(
    routes: &[PhysicalSignalAspectBindingDigest],
    mailboxes: &[Arc<PhysicalSignalRouteMailbox>],
) -> Box<[PhysicalSignalRouteOwner]> {
    routes
        .iter()
        .copied()
        .zip(mailboxes.iter())
        .map(|(route, mailbox)| PhysicalSignalRouteOwner {
            route,
            mailbox: Arc::clone(mailbox),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

#[expect(
    clippy::disallowed_methods,
    reason = "owner-thread thread-builder=1: Store owns one Signal graph owner thread for serialized graph mutation."
)]
fn spawn_graph_worker(
    boot: PhysicalSignalWorkerBoot,
) -> Result<JoinHandle<()>, PhysicalSignalConstructionFailure> {
    std::thread::Builder::new()
        .name("worth-store-physical-signal-graph".to_owned())
        .spawn(move || run_graph_worker_boot(boot))
        .map_err(|_| PhysicalSignalConstructionFailure::WorkerSpawnRejected)
}

fn run_graph_worker_boot(boot: PhysicalSignalWorkerBoot) {
    let liveness = PhysicalSignalWorkerLivenessGuard {
        admission: boot.shared.admission.clone(),
        mailboxes: boot.mailboxes,
        abandonments: boot.abandonments,
    };
    let result = PhysicalSignalGraph::build(boot.bindings);
    let Ok(mut graph) = result else {
        let _ = boot.ready.send(result.map(|_| ()));
        return;
    };
    *boot
        .shared
        .clock
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = graph.clock_basis();
    if boot.ready.send(Ok(())).is_err() {
        return;
    }
    run_graph_worker(&mut graph, &liveness, &boot.shared);
}

fn await_graph_readiness(
    ready: &mpsc::Receiver<Result<(), PhysicalSignalConstructionFailure>>,
    worker: JoinHandle<()>,
) -> Result<JoinHandle<()>, PhysicalSignalConstructionFailure> {
    match ready.recv() {
        Ok(Ok(())) => Ok(worker),
        Ok(Err(failure)) => {
            let _ = worker.join();
            Err(failure)
        }
        Err(_) => {
            let _ = worker.join();
            Err(PhysicalSignalConstructionFailure::WorkerReadinessLost)
        }
    }
}

fn run_graph_worker(
    graph: &mut PhysicalSignalGraph,
    liveness: &PhysicalSignalWorkerLivenessGuard,
    shared: &PhysicalSignalWorkerShared,
) {
    let mut next_route = 0;
    let mut prefer_abandonment = true;
    loop {
        if shared.stopping.load(Ordering::Acquire) {
            break;
        }
        if prefer_abandonment {
            match apply_next_abandonment(graph, &liveness.abandonments, shared) {
                Ok(true) => {
                    prefer_abandonment = false;
                    continue;
                }
                Ok(false) => {}
                Err(_) => break,
            }
        }
        if let Some((route_slot, command)) = next_command(&liveness.mailboxes, next_route) {
            next_route = route_slot.saturating_add(1);
            #[cfg(feature = "certification-test-authority")]
            pause_after_dequeue_for_certification(shared);
            if !graph.apply_command(route_slot, command) {
                break;
            }
            *shared
                .clock
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = graph.clock_basis();
            prefer_abandonment = true;
            continue;
        }
        match apply_next_abandonment(graph, &liveness.abandonments, shared) {
            Ok(true) => {
                prefer_abandonment = false;
                continue;
            }
            Ok(false) => {}
            Err(_) => break,
        }
        shared.wake.wait(&shared.stopping);
    }
}

#[cfg(feature = "certification-test-authority")]
fn pause_after_dequeue_for_certification(shared: &PhysicalSignalWorkerShared) {
    let gate = shared
        .pause_after_dequeue
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .as_ref()
        .map(super::CertificationPhysicalSignalPauseGate::worker_handle);
    if let Some(gate) = gate {
        gate.arrive_and_wait();
    }
}

fn apply_next_abandonment(
    graph: &mut PhysicalSignalGraph,
    abandonments: &PhysicalWorkAbandonmentInbox,
    _shared: &PhysicalSignalWorkerShared,
) -> Result<bool, PhysicalSignalAbandonmentFailure> {
    let Some(abandonment) = abandonments.pop() else {
        return Ok(false);
    };
    #[cfg(feature = "certification-test-authority")]
    pause_after_dequeue_for_certification(_shared);
    #[cfg(feature = "certification-test-authority")]
    if _shared.fail_next_abandonment.swap(false, Ordering::AcqRel) {
        return Err(PhysicalSignalAbandonmentFailure::InjectedWorkerFailure);
    }
    graph.abandon_work(
        abandonment.identity(),
        abandonment.route(),
        abandonment.consumer(),
    )?;
    abandonment.complete();
    Ok(true)
}

fn next_command(
    mailboxes: &[Arc<PhysicalSignalRouteMailbox>],
    next_route: usize,
) -> Option<(usize, PhysicalSignalRouteCommand)> {
    if mailboxes.is_empty() {
        return None;
    }
    for offset in 0..mailboxes.len() {
        let slot = next_route.wrapping_add(offset) % mailboxes.len();
        if let Some(command) = mailboxes[slot].pop() {
            return Some((slot, command));
        }
    }
    None
}
