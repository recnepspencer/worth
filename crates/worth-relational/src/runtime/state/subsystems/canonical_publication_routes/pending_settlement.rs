use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::history::data::CommitId;

use super::CanonicalPublicationRoute;

/// Derived candidates for settlement admission. Route atomics decide whether a
/// candidate has performed and remains unsettled; membership is not authority.
/// Registration precedes publication, so settlement cannot race a late insert.
#[derive(Default)]
pub(super) struct PendingSettlementRoutes {
    routes: Mutex<BTreeMap<CommitId, Arc<CanonicalPublicationRoute>>>,
}

impl PendingSettlementRoutes {
    pub(super) fn register(&self, route: Arc<CanonicalPublicationRoute>) {
        let displaced = self
            .routes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(route.commit.commit_id, route);
        // Never destroy route payloads while holding the index lock.
        drop(displaced);
    }

    pub(super) fn withdraw(&self, route: &Arc<CanonicalPublicationRoute>) {
        let removed = {
            let mut routes = self
                .routes
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if routes
                .get(&route.commit.commit_id)
                .is_some_and(|existing| Arc::ptr_eq(existing, route))
            {
                routes.remove(&route.commit.commit_id)
            } else {
                None
            }
        };
        drop(removed);
    }

    /// Visit only active reservations, in commit identity order. Settled history
    /// contributes no candidates; an unperformed reservation is invisible.
    pub(super) fn first_unsettled_commit(&self, visits: &AtomicU64) -> Option<CommitId> {
        let routes = self
            .routes
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        routes.iter().find_map(|(commit_id, route)| {
            visits.fetch_add(1, Ordering::Relaxed);
            (route.is_performed() && !route.is_settled()).then_some(*commit_id)
        })
    }
}
