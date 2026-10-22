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
            return Err(LeaseDenial::refused(
                LeaseDenial::EquivalenceContractUnavailable,
            ));
        }
        if request.policy.max_workers().get() > self.node.max_workers {
            return Err(LeaseDenial::refused(LeaseDenial::WorkerLimitExceedsParent));
        }
        if self.node.charged_memory_bytes.is_some_and(|parent| {
            request
                .policy
                .accounting()
                .and_then(|budget| budget.charged_memory_bytes())
                .is_some_and(|child| child > parent)
        }) {
            return Err(LeaseDenial::refused(LeaseDenial::MemoryLimitExceedsParent));
        }
        if self
            .policy
            .accounting()
            .and_then(|budget| budget.work_ceiling())
            .is_some_and(|parent| {
                request
                    .policy
                    .accounting()
                    .and_then(|budget| budget.work_ceiling())
                    .is_none_or(|child| child > parent)
            })
        {
            return Err(LeaseDenial::refused(LeaseDenial::WorkLimitExceedsParent));
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
        Self {
            authority: Arc::clone(&self.authority),
            _authority: std::marker::PhantomData,
            node: Arc::new(LeaseNode {
                accounted: request.policy.accounting().is_some(),
                id: self.authority.next_id(),
                parent: Some(Arc::clone(&self.node)),
                max_workers: request.policy.max_workers().get(),
                posture: request.policy.posture(),
                charged_memory_bytes: request
                    .policy
                    .accounting()
                    .and_then(|budget| budget.charged_memory_bytes()),
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

    /// An accounted ancestor cannot be bypassed by a worker-only descendant.
    pub fn has_accounting(&self) -> bool {
        self.policy.accounting().is_some()
            || self.authority.config.charged_memory_bytes.is_some()
            || std::iter::successors(Some(self.node.as_ref()), |node| node.parent.as_deref())
                .any(|node| node.accounted)
            || (self.current_descendant().is_some()
                && crate::backend::RunLimits::parent_accounted())
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
        if !crate::backend::HAS_OS_THREADS {
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
