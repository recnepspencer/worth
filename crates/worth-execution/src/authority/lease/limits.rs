use std::{sync::Arc, time::Instant};

use worth_foundational::{ExecutionFallbackCause, ExecutionPosture, ExecutionRequestPolicy};

use super::{
    CancellationToken, ExecutionLeaseStatus, ExecutionResourceLease, LeaseDenial, LeaseNode,
    LeaseRequest,
};

impl<'a> ExecutionResourceLease<'a> {
    pub fn status(&self) -> ExecutionLeaseStatus {
        ExecutionLeaseStatus {
            node: Arc::clone(&self.node),
        }
    }
    pub fn child(&self, request: LeaseRequest) -> Result<Self, LeaseDenial> {
        if request.policy.determinism() != self.policy.determinism() {
            return Err(LeaseDenial::EquivalenceContractUnavailable);
        }
        let budget = request.policy.budget();
        if budget.max_workers().get() > self.node.max_workers {
            return Err(LeaseDenial::WorkerLimitExceedsParent);
        }
        if budget.charged_memory_bytes() > self.node.charged_memory_bytes {
            return Err(LeaseDenial::MemoryLimitExceedsParent);
        }
        if budget.work_ceiling() > self.policy.budget().work_ceiling() {
            return Err(LeaseDenial::WorkLimitExceedsParent);
        }
        Ok(self.child_from_validated_request(request))
    }

    /// Creates an independently controlled child with this lease's exact policy.
    /// The parent remains unchanged; its cancellation and earlier deadline still
    /// constrain the child, and memory reservations charge the same lineage.
    pub fn controlled_child(
        &self,
        cancellation: CancellationToken,
        deadline: Option<Instant>,
    ) -> Self {
        self.child_from_validated_request(LeaseRequest {
            policy: self.policy,
            deadline,
            cancellation,
        })
    }

    fn child_from_validated_request(&self, request: LeaseRequest) -> Self {
        let budget = request.policy.budget();
        Self {
            authority: self.authority,
            node: Arc::new(LeaseNode {
                id: self.authority.next_id(),
                parent: Some(Arc::clone(&self.node)),
                max_workers: budget.max_workers().get(),
                posture: request.policy.posture(),
                charged_memory_bytes: budget.charged_memory_bytes(),
                deadline: match (self.node.deadline, request.deadline) {
                    (Some(parent), Some(child)) => Some(parent.min(child)),
                    (Some(parent), None) => Some(parent),
                    (None, child) => child,
                },
                cancellation: request.cancellation,
            }),
            policy: request.policy,
        }
    }

    pub fn policy(&self) -> &ExecutionRequestPolicy {
        &self.policy
    }

    pub fn resolved_posture(&self) -> ExecutionPosture {
        if self.serial_cause().is_some() {
            ExecutionPosture::Serial
        } else {
            ExecutionPosture::Automatic
        }
    }

    /// Why the lease resolves serial, if it does: the platform, a serial
    /// posture anywhere in its lineage, or a single worker.
    pub(crate) fn serial_cause(&self) -> Option<ExecutionFallbackCause> {
        if cfg!(target_arch = "wasm32") {
            Some(ExecutionFallbackCause::PlatformSerial)
        } else if self
            .lineage()
            .iter()
            .any(|node| node.posture == ExecutionPosture::Serial)
        {
            Some(ExecutionFallbackCause::PolicySerial)
        } else if self.node.max_workers == 1 {
            Some(ExecutionFallbackCause::WorkerLimit)
        } else {
            None
        }
    }

    pub fn is_cancelled(&self) -> bool {
        let mut node = Some(self.node.as_ref());
        while let Some(current) = node {
            if current.cancellation.is_cancelled() {
                return true;
            }
            node = current.parent.as_deref();
        }
        false
    }

    pub fn deadline_elapsed(&self) -> bool {
        self.node
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
    }

    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.node.deadline
    }

    pub(crate) fn cancellation_lineage(&self) -> impl Iterator<Item = CancellationToken> + '_ {
        std::iter::successors(Some(self.node.as_ref()), |node| node.parent.as_deref())
            .map(|node| node.cancellation.clone())
    }
}

impl ExecutionLeaseStatus {
    pub(crate) fn stop(&self) -> Option<crate::backend::KernelStop> {
        use crate::backend::KernelStop;
        if self.is_cancelled() {
            Some(KernelStop::Cancelled)
        } else if self.deadline_elapsed() {
            Some(KernelStop::DeadlineElapsed)
        } else {
            None
        }
    }

    pub fn is_cancelled(&self) -> bool {
        let mut node = Some(self.node.as_ref());
        while let Some(current) = node {
            if current.cancellation.is_cancelled() {
                return true;
            }
            node = current.parent.as_deref();
        }
        false
    }

    pub fn deadline_elapsed(&self) -> bool {
        self.node
            .deadline
            .is_some_and(|deadline| Instant::now() >= deadline)
    }
}
