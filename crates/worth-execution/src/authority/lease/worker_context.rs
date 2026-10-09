use std::{cell::RefCell, sync::Arc};

use super::{ActiveWorkerGuard, ExecutionResourceLease};

thread_local! {
    pub(super) static ACTIVE_WORKER: RefCell<Vec<(usize, u64)>> = const { RefCell::new(Vec::new()) };
}

impl ExecutionResourceLease<'_> {
    pub(crate) fn enter_worker_context(&self) -> ActiveWorkerGuard {
        let identity = Arc::as_ptr(&self.authority.inner) as usize;
        ACTIVE_WORKER.with(|active| active.borrow_mut().push((identity, self.node.id)));
        ActiveWorkerGuard
    }

    pub(crate) fn is_current_worker_in_lineage(&self) -> bool {
        let identity = Arc::as_ptr(&self.authority.inner) as usize;
        let active = ACTIVE_WORKER.with(|workers| workers.borrow().last().copied());
        active.is_some_and(|(owner, node)| {
            owner == identity && self.lineage().iter().any(|member| member.id == node)
        })
    }

    /// A prepared memory ticket does not reserve a worker yet. It must still
    /// reject a sibling lease's active worker before it retains memory.
    pub(crate) fn accepts_current_worker_context(&self) -> bool {
        let identity = Arc::as_ptr(&self.authority.inner) as usize;
        ACTIVE_WORKER.with(|workers| match workers.borrow().last().copied() {
            None => true,
            Some((owner, node)) => {
                owner == identity && self.lineage().iter().any(|member| member.id == node)
            }
        })
    }
}
