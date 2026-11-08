//! Receiver-armed read-only capture of genuine applied resolution inputs.
use super::*;
use std::{cell::RefCell, rc::Rc};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
type Read = (
    String,
    RelationalPinnedRead,
    RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
);
struct Pending {
    runtime: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    reads: Rc<RefCell<Vec<Read>>>,
}
thread_local! {
    static PENDING: RefCell<Option<Pending>> = const { RefCell::new(None) };
}

pub struct AppliedReadObservationForTest {
    reads: Rc<RefCell<Vec<Read>>>,
}
impl WorthQueryInstalledEntityResolutionContext {
    #[doc(hidden)]
    pub fn observe_applied_reads_for_test(&self) -> AppliedReadObservationForTest {
        let reads = Rc::new(RefCell::new(Vec::new()));
        PENDING.with(|pending| {
            let mut pending = pending.borrow_mut();
            assert!(pending.is_none(), "one applied-read recorder per thread");
            *pending = Some(Pending {
                runtime: self.runtime_authority,
                reads: Rc::clone(&reads),
            });
        });
        AppliedReadObservationForTest { reads }
    }
}
impl AppliedReadObservationForTest {
    pub fn take_reads_for_test(&self) -> Vec<Read> {
        std::mem::take(&mut *self.reads.borrow_mut())
    }
}
impl Drop for AppliedReadObservationForTest {
    fn drop(&mut self) {
        PENDING.with(|pending| {
            let mut pending = pending.borrow_mut();
            if pending
                .as_ref()
                .is_some_and(|p| Rc::ptr_eq(&p.reads, &self.reads))
            {
                *pending = None;
            }
        });
    }
}
pub(super) fn record(
    installed: &WorthQueryInstalledEntityResolutionContext,
    entity: &str,
    before: &RelationalPinnedRead,
    candidate: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
) {
    PENDING.with(|pending| {
        if let Some(pending) = pending.borrow().as_ref() {
            if pending.runtime == installed.runtime_authority {
                pending
                    .reads
                    .borrow_mut()
                    .push((entity.into(), before.clone(), candidate.clone()));
            }
        }
    });
}
