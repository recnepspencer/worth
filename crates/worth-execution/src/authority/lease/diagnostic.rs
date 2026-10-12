//! A sealed diagnostic shares the physical pool, never a request's ledger.
use super::*;
use worth_foundational::ExecutionBudget;

pub(crate) struct DiagnosticResourcesForTest {
    authority: ExecutionAuthority,
    node: Arc<LeaseNode>,
    policy: ExecutionRequestPolicy,
}

impl ExecutionResourceLease<'_> {
    pub(crate) fn diagnostic_resources_for_test(&self) -> DiagnosticResourcesForTest {
        fn detached(node: &LeaseNode) -> Arc<LeaseNode> {
            Arc::new(LeaseNode {
                id: node.id,
                parent: node.parent.as_deref().map(detached),
                max_workers: node.max_workers,
                posture: node.posture,
                charged_memory_bytes: u64::MAX,
                deadline: node.deadline,
                cancellation: node.cancellation.clone(),
            })
        }
        let policy = ExecutionRequestPolicy::new(
            self.policy.posture(),
            self.policy.determinism(),
            ExecutionBudget::new(self.policy.budget().max_workers(), u64::MAX, u64::MAX),
        );
        DiagnosticResourcesForTest {
            authority: ExecutionAuthority {
                inner: Arc::new(AuthorityInner {
                    config: ExecutionAuthorityConfig {
                        charged_memory_bytes: None,
                        ..self.authority.inner.config
                    },
                    // The one process pool is shared; no threads or authority are constructed.
                    pool: self.authority.inner.pool.clone(),
                    equivalences: self.authority.inner.equivalences.clone(),
                    ledger: Mutex::new(Ledger {
                        next_id: self.lineage().iter().map(|node| node.id).max().unwrap(),
                        ..Ledger::default()
                    }),
                }),
            },
            node: detached(&self.node),
            policy,
        }
    }
}
impl DiagnosticResourcesForTest {
    pub(crate) fn lease_for_test(&self) -> ExecutionResourceLease<'_> {
        ExecutionResourceLease {
            authority: &self.authority,
            node: Arc::clone(&self.node),
            policy: self.policy,
        }
    }
}
