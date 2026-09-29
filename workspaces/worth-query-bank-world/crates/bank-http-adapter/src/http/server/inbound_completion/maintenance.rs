//! One serial, cue-driven custodian owned by the installed Bank rail route.

use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tokio::sync::{watch, Notify};
use tokio::task::JoinHandle;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::route::BankRailCompletionRoute;
#[cfg(test)]
use super::route::BankRailMaintenanceBatch;

// Each host cue may traverse finite retained custody a few times to complete
// staged recovery. A persistent World/storage failure then parks until an
// external owner change or explicit host continuation cues another attempt.
const MAXIMUM_AUTOMATIC_SWEEPS_PER_CUE: usize = 4;

pub(in crate::http::server) fn start_maintenance(
    route: Arc<dyn BankRailCompletionRoute>,
    mut shutdown: watch::Receiver<bool>,
    wake: Arc<Notify>,
    maximum_deadline: Duration,
) -> JoinHandle<io::Result<()>> {
    tokio::spawn(async move {
        let mut next_expiry = None;
        loop {
            tokio::select! {
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() { return Ok(()); }
                }
                _ = wake.notified() => {}
                _ = async {
                    match next_expiry {
                        Some(expiry) => tokio::time::sleep_until(expiry).await,
                        None => std::future::pending::<()>().await,
                    }
                } => { next_expiry = None; }
            }
            if *shutdown.borrow() {
                return Ok(());
            }
            let mut unexamined: Option<(usize, usize)> = None;
            let mut sweep_progress = false;
            let mut sweeps = 1;
            loop {
                if *shutdown.borrow() {
                    return Ok(());
                }
                let Some(deadline) = Instant::now().checked_add(maximum_deadline) else {
                    return Err(io::Error::other("rail maintenance deadline overflow"));
                };
                let route = Arc::clone(&route);
                let batch = tokio::task::spawn_blocking(move || {
                    let cancellation = WorthQueryCancellationSource::new();
                    let request = WorthQueryRequestScope::new(deadline, cancellation.token());
                    route.maintain_custody(&request)
                })
                .await
                .map_err(io::Error::other)?;
                let Ok(report) = batch else { break };
                let made_progress =
                    report.performed > 0 || report.reclaimed > 0 || report.advanced > 0;
                sweep_progress |= made_progress;
                let left = match &mut unexamined {
                    Some((signed, transport)) => {
                        *signed = signed.saturating_sub(report.signed_selected);
                        *transport = transport.saturating_sub(report.transport_selected);
                        *signed + *transport
                    }
                    None => {
                        let signed = report
                            .signed_available_before
                            .saturating_sub(report.signed_selected);
                        let transport = report
                            .transport_available_before
                            .saturating_sub(report.transport_selected);
                        unexamined = Some((signed, transport));
                        signed + transport
                    }
                };
                next_expiry = report.next_expiry_unix_seconds.and_then(expiry_instant);
                if !made_progress
                    && report.blocked > 0
                    && next_expiry.is_some_and(|expiry| expiry <= tokio::time::Instant::now())
                {
                    next_expiry = None;
                }
                if left > 0 && report.selected > 0 {
                    continue;
                }
                if sweep_progress
                    && report.remaining > 0
                    && sweeps < MAXIMUM_AUTOMATIC_SWEEPS_PER_CUE
                {
                    sweeps += 1;
                    unexamined = None;
                    sweep_progress = false;
                    continue;
                }
                break;
            }
        }
    })
}

fn expiry_instant(expiry_unix_seconds: u64) -> Option<tokio::time::Instant> {
    let expiry =
        UNIX_EPOCH.checked_add(Duration::from_secs(expiry_unix_seconds.checked_add(1)?))?;
    let delay = expiry
        .duration_since(SystemTime::now())
        .unwrap_or(Duration::ZERO);
    tokio::time::Instant::now().checked_add(delay)
}

#[cfg(test)]
mod tests {
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
                signed_available_before: 5,
                signed_selected: 4,
                blocked: 4,
                remaining: 5,
                ..Default::default()
            },
            BankRailMaintenanceBatch {
                selected: 1,
                signed_available_before: 5,
                signed_selected: 1,
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
            signed_available_before: 1,
            signed_selected: 1,
            transport_available_before: 100,
            transport_selected: 2,
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

    #[tokio::test]
    async fn successful_intermediate_recovery_cues_next_ready_stage() {
        let route = Arc::new(ScriptedRoute::with_batches([
            BankRailMaintenanceBatch {
                selected: 1,
                signed_available_before: 1,
                signed_selected: 1,
                advanced: 1,
                remaining: 1,
                ..Default::default()
            },
            BankRailMaintenanceBatch {
                selected: 1,
                signed_available_before: 1,
                signed_selected: 1,
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
            signed_available_before: 1,
            signed_selected: 1,
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
}
