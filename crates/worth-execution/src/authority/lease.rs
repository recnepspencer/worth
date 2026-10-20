use std::{
    collections::HashMap,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
    time::Instant,
};

use rayon::ThreadPool;
use worth_foundational::{
    DeterminismContract, EquivalenceContractId, ExecutionPosture, ExecutionRequestPolicy,
};

use super::{
    equivalence::EquivalenceRegistry, CancellationToken, EquivalencePredicate,
    ExecutionMemoryReservation, MemoryLimitDenial,
};

mod array_backing;
mod byte_backing;
mod diagnostic;
mod fixed_backing;
mod limits;
mod memory;
mod process_authority;
mod refusal;
mod reservation;
mod retained;
mod room;
mod serial_authority;
mod serial_backing;
mod worker_context;
mod worker_width;
pub use array_backing::{ExecutionArray, ExecutionArrayBuilder, ExecutionArrayIntoIter};
pub use byte_backing::{ExecutionByteBuffer, ExecutionImmutableBytes};
pub use fixed_backing::{
    ExecutionAllocationDenial, ExecutionAllocationDenialClass, ExecutionAllocationDenialKind,
    ExecutionAllocationPolicy, ExecutionPhysicalAllocationDenialKind,
};
pub(in crate::authority) use memory::LeaseMemory;
pub(crate) use reservation::SlotRefusal;
use reservation::{ClaimSlot, RunInline};
use worker_context::ACTIVE_WORKER;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionAuthorityConfig {
    pub max_workers: NonZeroUsize,
    /// Optional process-wide payload-backing ceiling; finite request and ancestor ceilings also apply.
    pub charged_memory_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstructionDenial {
    AlreadyConstructed,
    PoolConstruction(String),
    DuplicateEquivalenceContract(EquivalenceContractId),
    DuplicateEquivalenceIdentity([u8; 32]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseDenial {
    WorkerLimitExceedsParent,
    MemoryLimitExceedsParent,
    WorkLimitExceedsParent,
    /// A memory limit on the process or the lease's lineage refused the bytes.
    MemoryExhausted(MemoryLimitDenial),
    /// The bytes a reservation would charge do not fit the charge counter.
    ChargedBytesOverflow,
    UnrelatedNestedLease,
    /// A lease-free reservation was requested outside an active serial scope.
    NoActiveExecutionScope,
    EquivalenceContractUnavailable,
}

/// Why an authority can never lease a policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionPolicyDenial {
    WorkersExceedAuthority,
    MemoryExceedsAuthority,
    ContractUnavailable,
}

impl From<ExecutionPolicyDenial> for LeaseDenial {
    fn from(denial: ExecutionPolicyDenial) -> Self {
        match denial {
            ExecutionPolicyDenial::WorkersExceedAuthority => Self::WorkerLimitExceedsParent,
            ExecutionPolicyDenial::MemoryExceedsAuthority => Self::MemoryLimitExceedsParent,
            ExecutionPolicyDenial::ContractUnavailable => Self::EquivalenceContractUnavailable,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LeaseRequest {
    pub policy: ExecutionRequestPolicy,
    pub deadline: Option<Instant>,
    pub cancellation: CancellationToken,
}

#[derive(Debug)]
pub struct ExecutionAuthority {
    inner: Arc<AuthorityInner>,
}

#[derive(Debug)]
struct AuthorityInner {
    config: ExecutionAuthorityConfig,
    pool: Option<Arc<ThreadPool>>,
    equivalences: EquivalenceRegistry,
    ledger: Mutex<Ledger>,
}

#[derive(Debug, Default)]
struct Ledger {
    next_id: u64,
    active_workers: usize,
    charged_memory_bytes: u64,
    nodes: HashMap<u64, NodeUsage>,
}

#[derive(Debug, Default)]
struct NodeUsage {
    active_workers: usize,
    charged_memory_bytes: u64,
}

#[derive(Debug)]
struct LeaseNode {
    id: u64,
    parent: Option<Arc<LeaseNode>>,
    max_workers: usize,
    posture: ExecutionPosture,
    charged_memory_bytes: Option<u64>,
    deadline: Option<Instant>,
    cancellation: CancellationToken,
}

#[derive(Debug)]
pub struct ExecutionResourceLease<'a> {
    authority: Arc<AuthorityInner>,
    _authority: std::marker::PhantomData<&'a ExecutionAuthority>,
    node: Arc<LeaseNode>,
    policy: ExecutionRequestPolicy,
}

/// Cloneable read-only cancellation/deadline lineage for long domain scans.
/// It grants no workers, memory, or independent work allowance.
#[derive(Debug, Clone)]
pub struct ExecutionLeaseStatus {
    node: Arc<LeaseNode>,
}

impl<'a> ExecutionResourceLease<'a> {
    pub(crate) fn equivalence_predicate(&self) -> Option<&EquivalencePredicate> {
        match self.policy.determinism() {
            DeterminismContract::CanonicalBitwise => None,
            DeterminismContract::ContractEquivalent(id) => self.authority.equivalences.get(id),
        }
    }

    fn lineage(&self) -> Vec<&LeaseNode> {
        let mut lineage = Vec::new();
        let mut node = Some(self.node.as_ref());
        while let Some(current) = node {
            lineage.push(current);
            node = current.parent.as_deref();
        }
        lineage
    }

    /// Reuse the invoking worker's slot for nested work. The child records one
    /// active worker against itself and descendants below the already-counted
    /// parent; newly retained memory is charged to every ancestor and process.
    /// An entry that finds no free slot runs inline on its own thread, so
    /// another run's occupancy never refuses it; only memory can.
    pub(crate) fn reserve_entry(
        &self,
        memory_bytes: u64,
    ) -> Result<ResourceReservation, LeaseDenial> {
        let identity = Arc::as_ptr(&self.authority) as usize;
        let active = ACTIVE_WORKER.with(|workers| {
            workers
                .borrow()
                .last()
                .map(|(owner, node, _)| (*owner, node.id))
        });
        match active {
            Some((owner, active_node)) if owner == identity => {
                let lineage = self.lineage();
                let Some(position) = lineage.iter().position(|node| node.id == active_node) else {
                    return Err(LeaseDenial::refused(LeaseDenial::UnrelatedNestedLease));
                };
                let worker_nodes = lineage[..position]
                    .iter()
                    .map(|node| node.id)
                    .collect::<Vec<_>>();
                self.reserve::<RunInline>(0, worker_nodes, 1, memory_bytes)
            }
            // An owned serial backing is still a root: a nested entry may
            // borrow only its invoking lease's lineage, never another root.
            Some(_) => Err(LeaseDenial::refused(LeaseDenial::UnrelatedNestedLease)),
            None => {
                let worker_nodes = self.lineage().iter().map(|node| node.id).collect();
                self.reserve::<RunInline>(1, worker_nodes, 1, memory_bytes)
            }
        }
    }

    pub(crate) fn try_reserve(
        &self,
        workers: usize,
        memory_bytes: u64,
    ) -> Result<ResourceReservation, SlotRefusal> {
        let worker_nodes = if workers == 0 {
            Vec::new()
        } else {
            self.lineage().iter().map(|node| node.id).collect()
        };
        self.reserve::<ClaimSlot>(workers, worker_nodes, workers, memory_bytes)
    }

    pub(crate) fn reserve_retained_memory(
        &self,
        memory_bytes: u64,
    ) -> Result<ResourceReservation, LeaseDenial> {
        self.reserve::<RunInline>(0, Vec::new(), 0, memory_bytes)
    }

    /// Transfer the same live backing charge atomically to this lease.
    /// A refusal keeps the original lineage and admitted bytes.
    pub fn transfer_memory(
        &self,
        ticket: &mut ExecutionMemoryReservation,
    ) -> Result<(), LeaseDenial> {
        let bytes = ticket.bytes();
        ticket.take_over(Some(self), None, bytes)
    }

    pub(crate) fn pool(&self) -> &ThreadPool {
        self.authority
            .pool
            .as_ref()
            .expect("native posture requires a native pool")
    }
}

pub(crate) struct ResourceReservation {
    authority: Arc<AuthorityInner>,
    memory_lineage: Vec<u64>,
    worker_lineage: Vec<u64>,
    worker_count_per_node: usize,
    process_workers: usize,
    memory_bytes: u64,
}

pub(crate) struct ActiveWorkerGuard;

impl Drop for ActiveWorkerGuard {
    fn drop(&mut self) {
        ACTIVE_WORKER.with(|active| {
            active.borrow_mut().pop().expect("balanced worker context");
        });
    }
}

impl Drop for ResourceReservation {
    fn drop(&mut self) {
        let mut ledger = self
            .authority
            .ledger
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        ledger.active_workers -= self.process_workers;
        ledger.charged_memory_bytes -= self.memory_bytes;
        for id in &self.memory_lineage {
            let usage = ledger.nodes.get_mut(id).expect("reservation has a node");
            usage.active_workers -= if self.worker_lineage.contains(id) {
                self.worker_count_per_node
            } else {
                0
            };
            usage.charged_memory_bytes -= self.memory_bytes;
            if usage.active_workers == 0 && usage.charged_memory_bytes == 0 {
                ledger.nodes.remove(id);
            }
        }
    }
}

impl AuthorityInner {
    fn next_id(&self) -> u64 {
        let mut ledger = self
            .ledger
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        ledger.next_id += 1;
        ledger.next_id
    }
}
