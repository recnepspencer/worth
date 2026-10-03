use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};

use dashmap::DashMap;

use crate::history::data::{CanonicalCommitEnvelope, CommitId, PositionedCanonicalCommit};
use crate::identity::data::VersionId;
use crate::publication::patch::data::PatchStreamPosition;

#[path = "canonical_publication_routes/checkpoint_admission.rs"]
mod checkpoint_admission;
#[path = "canonical_publication_routes/performed_stream.rs"]
mod performed_stream;
#[path = "canonical_publication_routes/position_admission.rs"]
mod position_admission;

pub(crate) use performed_stream::PerformedCheckpointSelection;
pub use performed_stream::RelationalPatchPositionReservationCounters;
use performed_stream::{PerformedPublicationRecordError, PerformedPublicationStream};
pub(crate) use position_admission::{
    readmit_positioned_canonical_commit, CanonicalPositionAdmission,
};

pub(crate) struct RelationalCanonicalPublicationRoutes {
    by_commit: DashMap<CommitId, Arc<CanonicalPublicationRoute>>,
    by_version: DashMap<VersionId, Arc<CanonicalPublicationRoute>>,
    performed_stream: PerformedPublicationStream,
    lifecycle: RwLock<()>,
}

impl std::fmt::Debug for RelationalCanonicalPublicationRoutes {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelationalCanonicalPublicationRoutes")
            .field("reserved_commit_count", &self.by_commit.len())
            .field("reserved_version_count", &self.by_version.len())
            .finish_non_exhaustive()
    }
}

struct CanonicalPublicationRoute {
    envelope: Arc<CanonicalCommitEnvelope>,
    handoff: Mutex<Option<CanonicalPublicationHandoff>>,
    performed_position: AtomicU64,
    performed: AtomicBool,
    settled: AtomicBool,
}

struct CanonicalPublicationHandoff {
    root: Arc<crate::branch::RelationalBranchRoot>,
    publication_cell: Option<crate::branch::RelationalBranchPublicationCell>,
}

pub(crate) enum CanonicalPublicationRecordError {
    ReservationContended,
    PositionCapacityExhausted,
}

pub(crate) enum CanonicalCheckpointAdmissionError {
    PublicationInFlight,
    PerformedPublicationRequiresSettlement(CommitId),
}

pub(crate) struct PreparedCanonicalPublicationRoute {
    routes: Arc<RelationalCanonicalPublicationRoutes>,
    route: Arc<CanonicalPublicationRoute>,
}

impl Default for RelationalCanonicalPublicationRoutes {
    fn default() -> Self {
        Self {
            by_commit: DashMap::new(),
            by_version: DashMap::new(),
            performed_stream: PerformedPublicationStream::default(),
            lifecycle: RwLock::new(()),
        }
    }
}

impl RelationalCanonicalPublicationRoutes {
    pub(crate) fn advance_position_floor(&self, floor: PatchStreamPosition) {
        self.performed_stream.advance_position_floor(floor);
    }

    pub(crate) fn install_recovered(
        &self,
        positioned: Arc<PositionedCanonicalCommit>,
    ) -> Result<(), &'static str> {
        let installs_version_route = positioned.envelope().authority_kind()
            != crate::history::data::CanonicalCommitAuthorityKind::BranchReferenceMovement;
        let route = Arc::new(CanonicalPublicationRoute {
            envelope: Arc::clone(positioned.canonical_arc()),
            handoff: Mutex::new(None),
            performed_position: AtomicU64::new(0),
            performed: AtomicBool::new(true),
            settled: AtomicBool::new(true),
        });
        if self
            .by_commit
            .insert(positioned.envelope().commit.commit_id, Arc::clone(&route))
            .is_some()
        {
            return Err("recovered canonical commit route is duplicated");
        }
        if installs_version_route
            && self
                .by_version
                .insert(positioned.envelope().commit.version_id, Arc::clone(&route))
                .is_some()
        {
            self.by_commit
                .remove(&positioned.envelope().commit.commit_id);
            return Err("recovered canonical version route is duplicated");
        }
        let position = positioned.position();
        self.performed_stream
            .link_recovered(positioned, Arc::clone(&route))?;
        route.set_position(position);
        Ok(())
    }

    pub(crate) fn reserve(
        routes: &Arc<Self>,
        envelope: Arc<CanonicalCommitEnvelope>,
        root: Arc<crate::branch::RelationalBranchRoot>,
    ) -> Result<PreparedCanonicalPublicationRoute, &'static str> {
        let route = Arc::new(CanonicalPublicationRoute {
            envelope,
            handoff: Mutex::new(Some(CanonicalPublicationHandoff {
                root,
                publication_cell: None,
            })),
            performed_position: AtomicU64::new(0),
            performed: AtomicBool::new(false),
            settled: AtomicBool::new(false),
        });
        let commit_id = route.envelope.commit.commit_id;
        let version_id = route.envelope.commit.version_id;
        if routes
            .by_commit
            .insert(commit_id, Arc::clone(&route))
            .is_some()
        {
            return Err("canonical publication commit identity is already reserved");
        }
        if routes
            .by_version
            .insert(version_id, Arc::clone(&route))
            .is_some()
        {
            routes.by_commit.remove(&commit_id);
            return Err("canonical publication version identity is already reserved");
        }
        Ok(PreparedCanonicalPublicationRoute {
            routes: Arc::clone(routes),
            route,
        })
    }

    pub(crate) fn by_commit(&self, commit_id: CommitId) -> Option<Arc<CanonicalCommitEnvelope>> {
        visible_envelope(self.by_commit.get(&commit_id).map(|entry| entry.clone()))
    }

    pub(crate) fn by_version(&self, version_id: VersionId) -> Option<Arc<CanonicalCommitEnvelope>> {
        visible_envelope(self.by_version.get(&version_id).map(|entry| entry.clone()))
    }

    pub(crate) fn by_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<CanonicalCommitEnvelope>> {
        self.performed_stream.envelope_at(position)
    }

    pub(crate) fn positioned_by_patch(
        &self,
        position: PatchStreamPosition,
    ) -> Option<Arc<PositionedCanonicalCommit>> {
        self.performed_stream.positioned_at(position)
    }

    pub(crate) fn visible_envelopes(&self) -> Vec<Arc<CanonicalCommitEnvelope>> {
        self.performed_stream.visible_envelopes()
    }

    pub(crate) fn latest_visible_patch(&self) -> Option<(PatchStreamPosition, CommitId)> {
        self.performed_stream.latest()
    }

    pub(crate) fn visible_patches_after(
        &self,
        after_position: Option<PatchStreamPosition>,
        max_commits: usize,
    ) -> Vec<(PatchStreamPosition, CommitId)> {
        self.performed_stream.after(after_position, max_commits)
    }

    pub(crate) fn stream_position(&self, commit_id: CommitId) -> Option<PatchStreamPosition> {
        let route = self
            .by_commit
            .get(&commit_id)
            .map(|entry| Arc::clone(entry.value()))?;
        route.is_visible().then(|| route.position()).flatten()
    }

    pub(crate) fn positioned_commit(
        &self,
        commit_id: CommitId,
    ) -> Option<Arc<PositionedCanonicalCommit>> {
        self.performed_stream.positioned(commit_id)
    }

    pub(crate) fn requires_settlement(&self, commit_id: CommitId) -> bool {
        self.by_commit
            .get(&commit_id)
            .is_some_and(|route| !route.settled.load(Ordering::Acquire))
    }

    pub(crate) fn enter_fork(
        &self,
    ) -> Result<RwLockWriteGuard<'_, ()>, crate::runtime::RelationalRuntimeForkDenial> {
        match self.lifecycle.try_write() {
            Ok(guard) => Ok(guard),
            Err(TryLockError::WouldBlock) => {
                Err(crate::runtime::RelationalRuntimeForkDenial::PublicationInFlight)
            }
            Err(TryLockError::Poisoned(poisoned)) => Ok(poisoned.into_inner()),
        }
    }

    pub(crate) fn performed_snapshot(&self) -> Vec<Arc<PositionedCanonicalCommit>> {
        self.performed_stream.positioned_snapshot()
    }

    pub(crate) fn reservation_counters(&self) -> RelationalPatchPositionReservationCounters {
        self.performed_stream.reservation_counters()
    }

    #[cfg(test)]
    pub(crate) fn pending_count(&self) -> usize {
        self.by_commit
            .iter()
            .filter(|route| !route.is_performed())
            .count()
    }

    #[cfg(test)]
    pub(crate) fn remove_for_test(&self, commit_id: CommitId) -> bool {
        let Some((_, route)) = self.by_commit.remove(&commit_id) else {
            return false;
        };
        self.by_version.remove(&route.envelope.commit.version_id);
        true
    }

    fn remove_unperformed(&self, route: &Arc<CanonicalPublicationRoute>) {
        if route.is_performed() {
            return;
        }
        self.by_commit
            .remove_if(&route.envelope.commit.commit_id, |_, existing| {
                Arc::ptr_eq(existing, route)
            });
        self.by_version
            .remove_if(&route.envelope.commit.version_id, |_, existing| {
                Arc::ptr_eq(existing, route)
            });
    }
}

impl PreparedCanonicalPublicationRoute {
    /// Owner-issued commit identity of the candidate this route publishes.
    /// Settlement is keyed by it before movement, so it must be readable
    /// without waiting for the route to perform.
    pub(crate) fn commit_id(&self) -> CommitId {
        self.route.envelope.commit.commit_id
    }

    pub(crate) fn selected_position(&self, commit_id: CommitId) -> Option<PatchStreamPosition> {
        self.routes.stream_position(commit_id)
    }

    pub(crate) fn enter_publication(&self) -> RwLockReadGuard<'_, ()> {
        self.routes
            .lifecycle
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn record_performed_with_cutover(
        &self,
        publication_cell: crate::branch::RelationalBranchPublicationCell,
        cutover: impl FnOnce(crate::publication::patch::data::PatchStreamPosition),
    ) -> Result<Arc<PositionedCanonicalCommit>, CanonicalPublicationRecordError> {
        self.route
            .handoff
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_mut()
            .expect("prepared route retains its handoff until movement")
            .publication_cell = Some(publication_cell);
        let positioned = self
            .routes
            .performed_stream
            .record_with_cutover(Arc::clone(&self.route), cutover)
            .map_err(|error| match error {
                PerformedPublicationRecordError::ReservationContended => {
                    CanonicalPublicationRecordError::ReservationContended
                }
                PerformedPublicationRecordError::PositionCapacityExhausted => {
                    CanonicalPublicationRecordError::PositionCapacityExhausted
                }
            })?;
        Ok(positioned)
    }
}

impl Drop for PreparedCanonicalPublicationRoute {
    fn drop(&mut self) {
        if !self.route.is_performed() {
            self.routes.remove_unperformed(&self.route);
        }
    }
}

impl CanonicalPublicationRoute {
    pub(super) fn set_position(&self, position: PatchStreamPosition) {
        self.performed_position.store(position.0, Ordering::Release);
    }

    fn position(&self) -> Option<PatchStreamPosition> {
        match self.performed_position.load(Ordering::Acquire) {
            0 => None,
            value => Some(PatchStreamPosition(value)),
        }
    }

    pub(super) fn is_performed(&self) -> bool {
        self.performed.load(Ordering::Acquire)
    }

    pub(super) fn mark_performed(&self) {
        self.performed.store(true, Ordering::Release);
        self.handoff
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take();
    }

    pub(super) fn is_settled(&self) -> bool {
        self.settled.load(Ordering::Acquire)
    }

    pub(super) fn mark_settled(&self) {
        self.settled.store(true, Ordering::Release);
    }

    pub(super) fn is_visible(&self) -> bool {
        if self.is_performed() {
            return true;
        }
        let handoff = self
            .handoff
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_ref()
            .and_then(|handoff| {
                handoff
                    .publication_cell
                    .as_ref()
                    .map(|cell| (cell.clone(), Arc::clone(&handoff.root)))
            });
        let handoff_visible =
            handoff.is_some_and(|(cell, root)| cell.currently_selects_root(&root));
        handoff_visible || self.is_performed()
    }
}

fn visible_envelope(
    route: Option<Arc<CanonicalPublicationRoute>>,
) -> Option<Arc<CanonicalCommitEnvelope>> {
    route.and_then(|route| route.is_visible().then(|| Arc::clone(&route.envelope)))
}
