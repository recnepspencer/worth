//! Type-erased shared handle for `PreparedRelationalCommitCandidate` payload.
//!
//! Uses unsized coercion on the existing `Arc<Mutex<Option<CandidatePayload>>>`
//! allocation so rustdoc auto-trait synthesis does not traverse commit-pipeline
//! execution internals through candidate handles.

use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, Mutex, Weak};

use super::CandidatePayload;

trait CandidatePayloadBacking:
    Send + Sync + std::panic::UnwindSafe + std::panic::RefUnwindSafe
{
    fn as_mutex(&self) -> &Mutex<Option<CandidatePayload>>;
}

impl CandidatePayloadBacking for Mutex<Option<CandidatePayload>> {
    fn as_mutex(&self) -> &Mutex<Option<CandidatePayload>> {
        self
    }
}

#[derive(Clone)]
pub(crate) struct SharedCandidatePayload(Arc<dyn CandidatePayloadBacking>);

#[derive(Clone)]
pub(crate) struct WeakCandidatePayload(Weak<dyn CandidatePayloadBacking>);

impl SharedCandidatePayload {
    pub(crate) fn new(payload: CandidatePayload) -> Self {
        Self(Arc::new(Mutex::new(Some(payload))))
    }

    pub(crate) fn downgrade(&self) -> WeakCandidatePayload {
        WeakCandidatePayload(Arc::downgrade(&self.0))
    }
}

impl Deref for SharedCandidatePayload {
    type Target = Mutex<Option<CandidatePayload>>;

    fn deref(&self) -> &Self::Target {
        self.0.as_mutex()
    }
}

impl WeakCandidatePayload {
    pub(crate) fn upgrade(&self) -> Option<SharedCandidatePayload> {
        self.0.upgrade().map(SharedCandidatePayload)
    }

    pub(crate) fn strong_count(&self) -> usize {
        self.0.strong_count()
    }
}

impl fmt::Debug for WeakCandidatePayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeakCandidatePayload")
            .field("strong_count", &self.0.strong_count())
            .finish()
    }
}
