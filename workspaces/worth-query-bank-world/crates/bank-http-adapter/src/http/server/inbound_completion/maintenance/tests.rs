use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

use worth_query_host::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundTerminalObservation,
};

use super::*;

#[derive(Default)]
struct CountingRoute {
    active: AtomicBool,
    batches: AtomicUsize,
}

#[derive(Default)]
struct ScriptedRoute {
    batches: Mutex<VecDeque<BankRailMaintenanceBatch>>,
    calls: AtomicUsize,
}

impl ScriptedRoute {
    fn with_batches(batches: impl IntoIterator<Item = BankRailMaintenanceBatch>) -> Self {
        Self {
            batches: Mutex::new(batches.into_iter().collect()),
            ..Self::default()
        }
    }
}

impl BankRailCompletionRoute for ScriptedRoute {
    fn receive_and_sign(
        &self,
        _: &[u8],
        _: &WorthQueryRequestScope,
    ) -> Result<(Vec<u8>, bool), WorthQueryInboundAdmissionDenial> {
        unreachable!("scripted maintenance never receives callbacks")
    }

    fn observe_terminal(&self, _: [u8; 32]) -> Option<WorthQueryInboundTerminalObservation> {
        None
    }

    fn maintain_custody(
        &self,
        _: &WorthQueryRequestScope,
    ) -> Result<BankRailMaintenanceBatch, WorthQueryInboundAdmissionDenial> {
        self.calls.fetch_add(1, Ordering::AcqRel);
        Ok(self
            .batches
            .lock()
            .expect("scripted batches lock")
            .pop_front()
            .unwrap_or_default())
    }
}

async fn wait_for_calls(route: &ScriptedRoute, count: usize) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while route.calls.load(Ordering::Acquire) < count {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("expected bounded maintenance continuation");
}

#[tokio::test]
async fn one_cue_sweeps_past_blocked_first_page() {
    let route = Arc::new(ScriptedRoute::with_batches([
        BankRailMaintenanceBatch {
            selected: 4,
            pools: [
                BankRailMaintenancePool {
                    signed_available_before: 5,
                    signed_selected: 4,
                    ..Default::default()
                },
                BankRailMaintenancePool::default(),
            ],
            blocked: 4,
            remaining: 5,
            ..Default::default()
        },
        BankRailMaintenanceBatch {
            selected: 1,
            pools: [
                BankRailMaintenancePool {
                    signed_available_before: 5,
                    signed_selected: 1,
                    ..Default::default()
                },
                BankRailMaintenancePool::default(),
            ],
            performed: 1,
            remaining: 4,
            ..Default::default()
        },
    ]));
    let (stop, receiver) = watch::channel(false);
    let wake = Arc::new(Notify::new());
    let task = start_maintenance(
        route.clone(),
        receiver,
        Arc::clone(&wake),
        Duration::from_secs(1),
    );
    wake.notify_one();
    wait_for_calls(&route, 2).await;
    stop.send_replace(true);
    task.await
        .expect("worker task should join")
        .expect("worker should stop");
}

#[tokio::test]
async fn one_cue_reaches_late_transport_work_despite_repeated_small_signed_pool() {
    let pages = (0..50).map(|page| BankRailMaintenanceBatch {
        selected: 3,
        pools: [
            BankRailMaintenancePool {
                signed_available_before: 1,
                signed_selected: 1,
                ..Default::default()
            },
            BankRailMaintenancePool {
                transport_available_before: 100,
                transport_selected: 2,
                ..Default::default()
            },
        ],
        blocked: if page == 49 { 2 } else { 3 },
        performed: usize::from(page == 49),
        remaining: 101 - usize::from(page == 49),
        ..Default::default()
    });
    let route = Arc::new(ScriptedRoute::with_batches(pages));
    let (stop, receiver) = watch::channel(false);
    let wake = Arc::new(Notify::new());
    let task = start_maintenance(
        route.clone(),
        receiver,
        Arc::clone(&wake),
        Duration::from_secs(1),
    );
    wake.notify_one();
    wait_for_calls(&route, 50).await;
    stop.send_replace(true);
    task.await
        .expect("worker task should join")
        .expect("worker should stop");
}

mod pool_balance;

#[tokio::test]
async fn successful_intermediate_recovery_cues_next_ready_stage() {
    let route = Arc::new(ScriptedRoute::with_batches([
        BankRailMaintenanceBatch {
            selected: 1,
            pools: [
                BankRailMaintenancePool {
                    signed_available_before: 1,
                    signed_selected: 1,
                    ..Default::default()
                },
                BankRailMaintenancePool::default(),
            ],
            advanced: 1,
            remaining: 1,
            ..Default::default()
        },
        BankRailMaintenanceBatch {
            selected: 1,
            pools: [
                BankRailMaintenancePool {
                    signed_available_before: 1,
                    signed_selected: 1,
                    ..Default::default()
                },
                BankRailMaintenancePool::default(),
            ],
            performed: 1,
            ..Default::default()
        },
    ]));
    let (stop, receiver) = watch::channel(false);
    let wake = Arc::new(Notify::new());
    let task = start_maintenance(
        route.clone(),
        receiver,
        Arc::clone(&wake),
        Duration::from_secs(1),
    );
    wake.notify_one();
    wait_for_calls(&route, 2).await;
    stop.send_replace(true);
    task.await
        .expect("worker task should join")
        .expect("worker should stop");
}

#[tokio::test]
async fn new_terminal_schedules_one_expiry_cleanup_without_polling() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("test wall clock follows Unix epoch")
        .as_secs();
    let route = Arc::new(ScriptedRoute::with_batches([
        BankRailMaintenanceBatch {
            next_expiry_unix_seconds: Some(now),
            ..Default::default()
        },
        BankRailMaintenanceBatch {
            reclaimed: 1,
            ..Default::default()
        },
    ]));
    let (stop, receiver) = watch::channel(false);
    let wake = Arc::new(Notify::new());
    let task = start_maintenance(
        route.clone(),
        receiver,
        Arc::clone(&wake),
        Duration::from_secs(1),
    );
    wake.notify_one();
    wait_for_calls(&route, 2).await;
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert_eq!(
        route.calls.load(Ordering::Acquire),
        2,
        "settled timer must not poll again"
    );
    stop.send_replace(true);
    task.await
        .expect("worker task should join")
        .expect("worker should stop");
}

#[tokio::test]
async fn persistent_republication_is_bounded_per_owner_cue() {
    let recurring = BankRailMaintenanceBatch {
        selected: 1,
        pools: [
            BankRailMaintenancePool {
                signed_available_before: 1,
                signed_selected: 1,
                ..Default::default()
            },
            BankRailMaintenancePool::default(),
        ],
        advanced: 1,
        remaining: 1,
        ..Default::default()
    };
    let route = Arc::new(ScriptedRoute::with_batches([recurring; 8]));
    let (stop, receiver) = watch::channel(false);
    let wake = Arc::new(Notify::new());
    let task = start_maintenance(
        route.clone(),
        receiver,
        Arc::clone(&wake),
        Duration::from_secs(1),
    );
    wake.notify_one();
    wait_for_calls(&route, MAXIMUM_AUTOMATIC_SWEEPS_PER_CUE).await;
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert_eq!(
        route.calls.load(Ordering::Acquire),
        MAXIMUM_AUTOMATIC_SWEEPS_PER_CUE
    );
    stop.send_replace(true);
    task.await
        .expect("worker task should join")
        .expect("worker should stop");
}

impl BankRailCompletionRoute for CountingRoute {
    fn receive_and_sign(
        &self,
        _: &[u8],
        _: &WorthQueryRequestScope,
    ) -> Result<(Vec<u8>, bool), WorthQueryInboundAdmissionDenial> {
        unreachable!("maintenance never receives callback bytes")
    }

    fn observe_terminal(&self, _: [u8; 32]) -> Option<WorthQueryInboundTerminalObservation> {
        None
    }

    fn maintain_custody(
        &self,
        _: &WorthQueryRequestScope,
    ) -> Result<BankRailMaintenanceBatch, WorthQueryInboundAdmissionDenial> {
        assert!(
            !self.active.swap(true, Ordering::AcqRel),
            "batches must be serial"
        );
        std::thread::sleep(Duration::from_millis(10));
        self.batches.fetch_add(1, Ordering::AcqRel);
        self.active.store(false, Ordering::Release);
        Err(WorthQueryInboundAdmissionDenial::RecoveryUnavailable)
    }
}

#[tokio::test]
async fn rail_maintenance_is_serial_and_ends_with_server_shutdown() {
    let route = Arc::new(CountingRoute::default());
    let (stop, receiver) = watch::channel(false);
    let wake = Arc::new(Notify::new());
    let task = start_maintenance(
        route.clone(),
        receiver,
        Arc::clone(&wake),
        Duration::from_secs(1),
    );
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert_eq!(route.batches.load(Ordering::Acquire), 0);
    wake.notify_one();
    tokio::time::timeout(Duration::from_secs(2), async {
        while route.batches.load(Ordering::Acquire) < 1 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the installed rail route should receive maintenance batches");
    wake.notify_one();
    tokio::time::timeout(Duration::from_secs(2), async {
        while route.batches.load(Ordering::Acquire) < 2 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the second cue should receive a serial maintenance batch");
    stop.send_replace(true);
    task.await
        .expect("maintenance task should join")
        .expect("orderly stop");
    let stopped_at = route.batches.load(Ordering::Acquire);
    tokio::time::sleep(Duration::from_millis(120)).await;
    assert_eq!(route.batches.load(Ordering::Acquire), stopped_at);
}
