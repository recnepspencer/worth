//! Immutable component Weak plus fixed Copy axes under the SAME lane mutex.
use super::*;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};
#[derive(Debug)]
pub(super) struct RelationalRetentionObligationRecord {
    pub(super) component: Option<Weak<crate::branch::AdmittedRelationalBranchBasisInner>>,
    pub(super) active_operation: bool,
    kind: AtomicU8,
    first: AtomicU64,
    second: AtomicU64,
    len: AtomicU8,
}
impl RelationalRetentionObligationRecord {
    pub(super) fn new(
        kind: RelationalRetentionObligationKind,
        component: Option<Weak<crate::branch::AdmittedRelationalBranchBasisInner>>,
        roots: roots::RootIds,
        active_operation: bool,
    ) -> Self {
        Self {
            component,
            active_operation,
            kind: AtomicU8::new(kind as u8),
            first: AtomicU64::new(roots[0]),
            second: AtomicU64::new(if roots.len() == 2 { roots[1] } else { 0 }),
            len: AtomicU8::new(roots.len() as u8),
        }
    }
    // All accesses are protected by the existing lane mutex. Atomic storage
    // preserves Send/Sync through the SDK's immutable Arc<Entry>; it introduces
    // no independent synchronization protocol or semantic counter.
    pub(super) fn kind(&self) -> RelationalRetentionObligationKind {
        use RelationalRetentionObligationKind::*;
        match self.kind.load(Ordering::Relaxed) {
            value if value == Observation as u8 => Observation,
            value if value == Transaction as u8 => Transaction,
            value if value == Candidate as u8 => Candidate,
            value if value == PerformedSettlement as u8 => PerformedSettlement,
            value if value == ExternalComponentBasis as u8 => ExternalComponentBasis,
            _ => unreachable!("private obligation kind is only written from the fixed enum"),
        }
    }
    pub(super) fn root_ids(&self) -> roots::RootIds {
        let first = self.first.load(Ordering::Relaxed);
        let mut result = roots::RootIds::one(first);
        if self.len.load(Ordering::Relaxed) == 2 {
            result = roots::RootIds::two(first, self.second.load(Ordering::Relaxed));
        }
        result
    }
    pub(super) fn transfer_settlement(&self, root: u64) {
        self.kind.store(
            RelationalRetentionObligationKind::PerformedSettlement as u8,
            Ordering::Relaxed,
        );
        self.first.store(root, Ordering::Relaxed);
        self.second.store(0, Ordering::Relaxed);
        self.len.store(1, Ordering::Relaxed);
    }
}
