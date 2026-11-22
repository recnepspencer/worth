use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_GRAPH_CALL_AUTHORITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct WorthQueryGraphCallAuthorityIdentity(u64);

impl WorthQueryGraphCallAuthorityIdentity {
    pub(super) fn mint() -> Self {
        Self(NEXT_GRAPH_CALL_AUTHORITY.fetch_add(1, Ordering::Relaxed))
    }

    pub(super) fn as_u64(self) -> u64 {
        self.0
    }
}
