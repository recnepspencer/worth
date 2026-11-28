use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard};

use super::BridgePreviewSessionIdentity;

#[derive(Debug)]
struct BridgePreviewSessionLivenessState {
    incarnation: u64,
    active: RwLock<bool>,
    session_identity: BridgePreviewSessionIdentity,
}

#[derive(Debug)]
pub(crate) struct BridgePreviewSessionLivenessOwner {
    state: Arc<BridgePreviewSessionLivenessState>,
}

/// Bridge-owned observation of one active preview-session lifecycle.
///
/// The observer does not keep the session alive and cannot perform preview
/// work. An admitted guard holds the lifecycle read boundary while a consumer
/// performs work already authorized by that session.
#[derive(Debug, Clone)]
pub struct BridgePreviewSessionLivenessObserver {
    state: Arc<BridgePreviewSessionLivenessState>,
}

pub struct BridgePreviewSessionLivenessGuard<'observer> {
    _liveness: RwLockReadGuard<'observer, bool>,
}

impl BridgePreviewSessionLivenessOwner {
    pub(crate) fn new(session_identity: BridgePreviewSessionIdentity) -> Self {
        static NEXT_INCARNATION: AtomicU64 = AtomicU64::new(1);
        Self {
            state: Arc::new(BridgePreviewSessionLivenessState {
                incarnation: NEXT_INCARNATION.fetch_add(1, Ordering::Relaxed),
                active: RwLock::new(true),
                session_identity,
            }),
        }
    }

    pub(super) fn session_identity(&self) -> &BridgePreviewSessionIdentity {
        &self.state.session_identity
    }

    pub(crate) fn observer(&self) -> BridgePreviewSessionLivenessObserver {
        BridgePreviewSessionLivenessObserver {
            state: Arc::clone(&self.state),
        }
    }
}

impl Drop for BridgePreviewSessionLivenessOwner {
    fn drop(&mut self) {
        match self.state.active.write() {
            Ok(mut active) => *active = false,
            Err(poisoned) => *poisoned.into_inner() = false,
        }
    }
}

impl BridgePreviewSessionLivenessObserver {
    pub(crate) fn incarnation(&self) -> u64 {
        self.state.incarnation
    }

    pub fn session_identity(&self) -> &BridgePreviewSessionIdentity {
        &self.state.session_identity
    }

    pub fn admit_active_session(&self) -> Option<BridgePreviewSessionLivenessGuard<'_>> {
        let liveness = self.state.active.read().ok()?;
        if !*liveness {
            return None;
        }
        Some(BridgePreviewSessionLivenessGuard {
            _liveness: liveness,
        })
    }
}

impl PartialEq for BridgePreviewSessionLivenessObserver {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }
}

impl Eq for BridgePreviewSessionLivenessObserver {}
