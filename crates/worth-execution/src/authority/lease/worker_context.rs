use std::{cell::RefCell, sync::Arc};

use super::{ActiveWorkerGuard, ExecutionResourceLease, LeaseNode};
use worth_foundational::ExecutionRequestPolicy;

thread_local! {
    pub(super) static ACTIVE_WORKER: RefCell<Vec<(usize, Arc<LeaseNode>, ExecutionRequestPolicy)>> = const { RefCell::new(Vec::new()) };
}

impl ExecutionResourceLease<'_> {
    pub(crate) fn enter_worker_context(&self) -> ActiveWorkerGuard {
        let identity = Arc::as_ptr(&self.authority) as usize;
        ACTIVE_WORKER.with(|active| {
            active
                .borrow_mut()
                .push((identity, Arc::clone(&self.node), self.policy))
        });
        ActiveWorkerGuard
    }

    pub(crate) fn is_current_worker_in_lineage(&self) -> bool {
        let identity = Arc::as_ptr(&self.authority) as usize;
        let active = ACTIVE_WORKER.with(|workers| workers.borrow().last().cloned());
        active.is_some_and(|(owner, node, _)| {
            owner == identity && self.lineage().iter().any(|member| member.id == node.id)
        })
    }

    /// A prepared memory ticket does not reserve a worker yet. It must still
    /// reject a sibling lease's active worker before it retains memory.
    pub(crate) fn accepts_current_worker_context(&self) -> bool {
        let identity = Arc::as_ptr(&self.authority) as usize;
        ACTIVE_WORKER.with(|workers| match workers.borrow().last().cloned() {
            None => true,
            Some((owner, node, _)) => {
                owner == identity && self.lineage().iter().any(|member| member.id == node.id)
            }
        })
    }
    /// Reborrow the already-active descendant of this request lease. No policy
    /// or account is created; sibling/foreign workers are never substituted.
    pub(in crate::authority) fn current_descendant(&self) -> Option<Self> {
        let identity = Arc::as_ptr(&self.authority) as usize;
        let active = ACTIVE_WORKER.with(|workers| workers.borrow().last().cloned());
        let (owner, node, policy) = active?;
        if owner != identity
            || !std::iter::successors(Some(node.as_ref()), |node| node.parent.as_deref())
                .any(|node| node.id == self.node.id)
        {
            return None;
        }
        Some(Self {
            authority: Arc::clone(&self.authority),
            _authority: std::marker::PhantomData,
            node,
            policy,
        })
    }
}
