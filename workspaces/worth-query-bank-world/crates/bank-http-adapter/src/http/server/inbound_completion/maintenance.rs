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
use super::route::{BankRailMaintenanceBatch, BankRailMaintenancePool};

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
            let mut unexamined: Option<[[usize; 2]; 2]> = None;
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
                let mut remaining = unexamined.take().unwrap_or_else(|| {
                    report.pools.map(|pool| {
                        [
                            pool.signed_available_before,
                            pool.transport_available_before,
                        ]
                    })
                });
                for (lanes, pool) in remaining.iter_mut().zip(report.pools) {
                    // A pool can disappear between batches (or its route can
                    // fail and report no available work). Park its old count
                    // rather than letting another pool's selections drain it.
                    lanes[0] = lanes[0]
                        .min(pool.signed_available_before)
                        .saturating_sub(pool.signed_selected);
                    lanes[1] = lanes[1]
                        .min(pool.transport_available_before)
                        .saturating_sub(pool.transport_selected);
                }
                let left = remaining
                    .iter()
                    .map(|lanes| lanes[0].saturating_add(lanes[1]))
                    .sum::<usize>();
                unexamined = Some(remaining);
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
mod tests;
