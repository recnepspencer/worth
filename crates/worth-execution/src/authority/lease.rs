use std::{
    cell::RefCell,
    collections::HashMap,
    num::NonZeroUsize,
    sync::{Arc, Mutex, OnceLock},
    time::Instant,
};

use rayon::{ThreadPool, ThreadPoolBuilder};
use worth_foundational::{
    DeterminismContract, EquivalenceContractId, ExecutionPosture, ExecutionRequestPolicy,
};

use super::{equivalence::EquivalenceRegistry, CancellationToken, EquivalencePredicate};

static PROCESS_AUTHORITY: OnceLock<()> = OnceLock::new();
static CONSTRUCTION_LOCK: Mutex<()> = Mutex::new(());
mod byte_backing;
mod limits;
mod memory_reservation;
mod retained;
mod worker_context;
pub use byte_backing::{
    ExecutionByteAllocationDenial, ExecutionByteAllocationDenialKind,
    ExecutionByteAllocationPolicy, ExecutionByteBuffer, ExecutionImmutableBytes,
};
pub use memory_reservation::ExecutionMemoryReservation;
thread_local! {
    static ACTIVE_WORKER: RefCell<Vec<(usize, u64)>> = const { RefCell::new(Vec::new()) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionAuthorityConfig {
    pub max_workers: NonZeroUsize,
    /// Optional process-wide payload-backing ceiling; request/ancestor bounds remain finite.
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
    ResourceExhausted,
    UnrelatedNestedLease,
    EquivalenceContractUnavailable,
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
    pool: Option<ThreadPool>,
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
    charged_memory_bytes: u64,
    deadline: Option<Instant>,
    cancellation: CancellationToken,
}

#[derive(Debug)]
pub struct ExecutionResourceLease<'a> {
    authority: &'a ExecutionAuthority,
    node: Arc<LeaseNode>,
    policy: ExecutionRequestPolicy,
}

/// Cloneable read-only cancellation/deadline lineage for long domain scans.
/// It grants no workers, memory, or independent work allowance.
#[derive(Debug, Clone)]
pub struct ExecutionLeaseStatus {
    node: Arc<LeaseNode>,
}

/// One physical authority is permitted for a process lifetime, including after drop.
impl ExecutionAuthority {
    pub fn try_construct(config: ExecutionAuthorityConfig) -> Result<Self, ConstructionDenial> {
        Self::try_construct_with_equivalences(config, [])
    }

    /// Installs an immutable set of comparison contracts at the composition root.
    pub fn try_construct_with_equivalences(
        config: ExecutionAuthorityConfig,
        predicates: impl IntoIterator<Item = EquivalencePredicate>,
    ) -> Result<Self, ConstructionDenial> {
        let _lock = CONSTRUCTION_LOCK
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if PROCESS_AUTHORITY.get().is_some() {
            return Err(ConstructionDenial::AlreadyConstructed);
        }
        let equivalences = EquivalenceRegistry::new(predicates)?;
        let pool = if cfg!(target_arch = "wasm32") {
            None
        } else {
            Some(
                ThreadPoolBuilder::new()
                    .num_threads(config.max_workers.get())
                    .thread_name(|index| format!("worth-execution-{index}"))
                    .build()
                    .map_err(|error| ConstructionDenial::PoolConstruction(error.to_string()))?,
            )
        };
        let authority = Self {
            inner: Arc::new(AuthorityInner {
                config,
                pool,
                equivalences,
                ledger: Mutex::new(Ledger::default()),
            }),
        };
        PROCESS_AUTHORITY
            .set(())
            .map_err(|()| ConstructionDenial::AlreadyConstructed)?;
        Ok(authority)
    }

    pub fn request_lease(
        &self,
        request: LeaseRequest,
    ) -> Result<ExecutionResourceLease<'_>, LeaseDenial> {
        if let DeterminismContract::ContractEquivalent(id) = request.policy.determinism() {
            if self.inner.equivalences.get(id).is_none() {
                return Err(LeaseDenial::EquivalenceContractUnavailable);
            }
        }
        let budget = request.policy.budget();
        if budget.max_workers().get() > self.inner.config.max_workers.get() {
            return Err(LeaseDenial::WorkerLimitExceedsParent);
        }
        if self
            .inner
            .config
            .charged_memory_bytes
            .is_some_and(|cap| budget.charged_memory_bytes() > cap)
        {
            return Err(LeaseDenial::MemoryLimitExceedsParent);
        }
        let id = self.next_id();
        Ok(ExecutionResourceLease {
            authority: self,
            node: Arc::new(LeaseNode {
                id,
                parent: None,
                max_workers: budget.max_workers().get(),
                posture: request.policy.posture(),
                charged_memory_bytes: budget.charged_memory_bytes(),
                deadline: request.deadline,
                cancellation: request.cancellation,
            }),
            policy: request.policy,
        })
    }

    fn next_id(&self) -> u64 {
        let mut ledger = self
            .inner
            .ledger
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        ledger.next_id += 1;
        ledger.next_id
    }
}

impl<'a> ExecutionResourceLease<'a> {
    pub(crate) fn equivalence_predicate(&self) -> Option<&EquivalencePredicate> {
        match self.policy.determinism() {
            DeterminismContract::CanonicalBitwise => None,
            DeterminismContract::ContractEquivalent(id) => {
                self.authority.inner.equivalences.get(id)
            }
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
    pub(crate) fn reserve_entry(
        &self,
        memory_bytes: u64,
    ) -> Result<ResourceReservation, LeaseDenial> {
        let identity = Arc::as_ptr(&self.authority.inner) as usize;
        let active = ACTIVE_WORKER.with(|workers| workers.borrow().last().copied());
        match active {
            Some((owner, active_node)) if owner == identity => {
                let lineage = self.lineage();
                let Some(position) = lineage.iter().position(|node| node.id == active_node) else {
                    return Err(LeaseDenial::UnrelatedNestedLease);
                };
                let worker_nodes = lineage[..position]
                    .iter()
                    .map(|node| node.id)
                    .collect::<Vec<_>>();
                self.reserve(0, worker_nodes, 1, memory_bytes)
            }
            _ => self.try_reserve(1, memory_bytes),
        }
    }

    pub(crate) fn try_reserve(
        &self,
        workers: usize,
        memory_bytes: u64,
    ) -> Result<ResourceReservation, LeaseDenial> {
        let worker_nodes = if workers == 0 {
            Vec::new()
        } else {
            self.lineage().iter().map(|node| node.id).collect()
        };
        self.reserve(workers, worker_nodes, workers, memory_bytes)
    }

    pub(crate) fn reserve_retained_memory(
        &self,
        memory_bytes: u64,
    ) -> Result<ResourceReservation, LeaseDenial> {
        self.reserve(0, Vec::new(), 0, memory_bytes)
    }

    fn reserve(
        &self,
        process_workers: usize,
        worker_nodes: Vec<u64>,
        worker_count_per_node: usize,
        memory_bytes: u64,
    ) -> Result<ResourceReservation, LeaseDenial> {
        if process_workers == 0 && worker_count_per_node == 0 && memory_bytes == 0 {
            return Ok(ResourceReservation {
                authority: Arc::clone(&self.authority.inner),
                memory_lineage: Vec::new(),
                worker_lineage: Vec::new(),
                worker_count_per_node: 0,
                process_workers: 0,
                memory_bytes: 0,
            });
        }
        let lineage = self.lineage();
        let mut ledger = self
            .authority
            .inner
            .ledger
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let workers_after = ledger.active_workers.checked_add(process_workers);
        let process_memory = ledger.charged_memory_bytes.checked_add(memory_bytes);
        if workers_after.is_none_or(|count| count > self.authority.inner.config.max_workers.get())
            || process_memory.is_none_or(|count| {
                self.authority
                    .inner
                    .config
                    .charged_memory_bytes
                    .is_some_and(|cap| count > cap)
            })
        {
            return Err(LeaseDenial::ResourceExhausted);
        }
        for node in &lineage {
            let usage = ledger.nodes.get(&node.id);
            let worker_increment = if worker_nodes.contains(&node.id) {
                worker_count_per_node
            } else {
                0
            };
            let workers_after = usage.map_or(Some(worker_increment), |value| {
                value.active_workers.checked_add(worker_increment)
            });
            let memory_after = usage.map_or(Some(memory_bytes), |value| {
                value.charged_memory_bytes.checked_add(memory_bytes)
            });
            if workers_after.is_none_or(|count| count > node.max_workers)
                || memory_after.is_none_or(|count| count > node.charged_memory_bytes)
            {
                return Err(LeaseDenial::ResourceExhausted);
            }
        }
        ledger.active_workers = workers_after.expect("checked above");
        ledger.charged_memory_bytes = process_memory.expect("checked above");
        for node in &lineage {
            let usage = ledger.nodes.entry(node.id).or_default();
            usage.active_workers += if worker_nodes.contains(&node.id) {
                worker_count_per_node
            } else {
                0
            };
            usage.charged_memory_bytes += memory_bytes;
        }
        Ok(ResourceReservation {
            authority: Arc::clone(&self.authority.inner),
            memory_lineage: lineage.iter().map(|node| node.id).collect(),
            worker_lineage: worker_nodes,
            worker_count_per_node,
            process_workers,
            memory_bytes,
        })
    }

    pub(crate) fn pool(&self) -> &ThreadPool {
        self.authority
            .inner
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
