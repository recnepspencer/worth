//! The process's physical authority and the policies it may lease.
use super::{
    AuthorityInner, ConstructionDenial, DeterminismContract, EquivalencePredicate,
    EquivalenceRegistry, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionPolicyDenial,
    ExecutionRequestPolicy, ExecutionResourceLease, LeaseDenial, LeaseNode, LeaseRequest, Ledger,
};
use rayon::ThreadPoolBuilder;
use std::sync::{Arc, Mutex, OnceLock};
static PROCESS_AUTHORITY: OnceLock<()> = OnceLock::new();
static CONSTRUCTION_LOCK: Mutex<()> = Mutex::new(());

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
        let pool = if !crate::backend::HAS_OS_THREADS {
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

    /// Whether this authority can ever lease `policy`: its contract is
    /// registered and its workers and memory fit the process. A refusal is
    /// the one [`Self::request_lease`] makes for every request of `policy`.
    pub fn admits_policy(
        &self,
        policy: &ExecutionRequestPolicy,
    ) -> Result<(), ExecutionPolicyDenial> {
        if let DeterminismContract::ContractEquivalent(id) = policy.determinism() {
            if self.inner.equivalences.get(id).is_none() {
                return Err(ExecutionPolicyDenial::ContractUnavailable);
            }
        }
        let budget = policy.budget();
        if budget.max_workers().get() > self.inner.config.max_workers.get() {
            return Err(ExecutionPolicyDenial::WorkersExceedAuthority);
        }
        if self
            .inner
            .config
            .charged_memory_bytes
            .is_some_and(|cap| budget.charged_memory_bytes() > cap)
        {
            return Err(ExecutionPolicyDenial::MemoryExceedsAuthority);
        }
        Ok(())
    }

    pub fn request_lease(
        &self,
        request: LeaseRequest,
    ) -> Result<ExecutionResourceLease<'_>, LeaseDenial> {
        self.admits_policy(&request.policy)
            .map_err(|denial| LeaseDenial::refused(denial.into()))?;
        let budget = request.policy.budget();
        let id = self.inner.next_id();
        Ok(ExecutionResourceLease {
            authority: Arc::clone(&self.inner),
            _authority: std::marker::PhantomData,
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
}
