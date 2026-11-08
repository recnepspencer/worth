//! Bounded retained slots issue weak references; expired identities need no tombstones.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, RwLock, Weak,
};
use worth_foundational::facade::{
    DiagnosticRecordAbsence as Absence, DiagnosticRecordReadDenial as Denial,
    DurableSignalOwnerIdentity,
};
#[cfg(test)]
mod tests;

#[derive(Debug, Default)]
pub(super) struct WritebackDiagnosticScope {
    pub(super) owner: Option<DurableSignalOwnerIdentity>,
}
/// An owner-issued reference observes diagnostic retention, never effect authority.
/// Keeping it does not keep the record payload or owner alive.
#[derive(Debug, Clone)]
pub struct BridgeWritebackDiagnosticReference<R> {
    origin: Weak<WritebackDiagnosticScope>,
    owner: Option<DurableSignalOwnerIdentity>,
    slot: Weak<WritebackDiagnosticSlot<R>>,
}
#[derive(Debug)]
pub(super) struct WritebackDiagnosticSlot<R> {
    value: RwLock<Arc<R>>,
    retained: AtomicBool,
}
impl<R> WritebackDiagnosticSlot<R> {
    pub(super) fn new(record: R) -> Arc<Self> {
        Arc::new(Self {
            value: RwLock::new(Arc::new(record)),
            retained: AtomicBool::new(true),
        })
    }
    pub(super) fn value(&self) -> Arc<R> {
        Arc::clone(
            &self
                .value
                .read()
                .expect("diagnostic slot contains only owned record data"),
        )
    }
    pub(super) fn replace(&self, record: R) {
        *self
            .value
            .write()
            .expect("diagnostic slot contains only owned record data") = Arc::new(record);
    }
    pub(super) fn expire(&self) {
        self.retained.store(false, Ordering::Release);
    }
    pub(super) fn reference(
        self: &Arc<Self>,
        scope: &Arc<WritebackDiagnosticScope>,
    ) -> BridgeWritebackDiagnosticReference<R> {
        BridgeWritebackDiagnosticReference {
            origin: Arc::downgrade(scope),
            owner: scope.owner,
            slot: Arc::downgrade(self),
        }
    }
}
impl WritebackDiagnosticScope {
    // Latest/list access names the current archive. Historical lookup needs an
    // issued reference; an unknown digest cannot establish a retention history.
    pub(super) fn missing(&self) -> Absence {
        Absence::NeverRetained
    }
    pub(super) fn read<R: Clone>(
        self: &Arc<Self>,
        reference: &BridgeWritebackDiagnosticReference<R>,
    ) -> Result<R, Denial> {
        let same = reference
            .origin
            .upgrade()
            .is_some_and(|origin| Arc::ptr_eq(&origin, self));
        if !same {
            return if self.owner.is_some() && self.owner == reference.owner {
                Err(Denial::Absent(Absence::NotRetainedAcrossReopen))
            } else {
                Err(Denial::ForeignOwner)
            };
        }
        let slot = reference
            .slot
            .upgrade()
            .ok_or(Denial::Absent(Absence::ExpiredUnderCountLimit))?;
        if !slot.retained.load(Ordering::Acquire) {
            return Err(Denial::Absent(Absence::ExpiredUnderCountLimit));
        }
        Ok(slot.value().as_ref().clone())
    }
}
