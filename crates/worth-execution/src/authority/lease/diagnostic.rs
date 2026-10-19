//! A sealed diagnostic shares the physical pool, never a request's ledger.
use super::*;
use worth_foundational::ExecutionBudget;

pub(crate) struct DiagnosticResourcesForTest {
    authority: ExecutionAuthority,
    node: Arc<LeaseNode>,
    policy: ExecutionRequestPolicy,
}

/// A diagnostic's fresh ledger cannot inherit the invoking worker's identity.
pub(crate) struct DetachedWorkerContext {
    previous: Vec<(usize, Arc<LeaseNode>, ExecutionRequestPolicy)>,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl Drop for DetachedWorkerContext {
    fn drop(&mut self) {
        ACTIVE_WORKER.with(|active| {
            debug_assert!(
                active.borrow().is_empty(),
                "diagnostic workers have settled"
            );
            *active.borrow_mut() = std::mem::take(&mut self.previous);
        });
    }
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
            ExecutionBudget::new(self.policy.budget().max_workers(), u64::MAX, None),
        );
        DiagnosticResourcesForTest {
            authority: ExecutionAuthority {
                inner: Arc::new(AuthorityInner {
                    config: ExecutionAuthorityConfig {
                        charged_memory_bytes: None,
                        ..self.authority.config
                    },
                    // The diagnostic owns a fresh ledger but constructs no worker threads.
                    pool: self.authority.pool.clone(),
                    equivalences: self.authority.equivalences.clone(),
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
    pub(crate) fn detach_worker_context(&self) -> DetachedWorkerContext {
        DetachedWorkerContext {
            previous: ACTIVE_WORKER.with(|active| std::mem::take(&mut *active.borrow_mut())),
            _thread: std::marker::PhantomData,
        }
    }

    pub(crate) fn lease_for_test(&self) -> ExecutionResourceLease<'_> {
        ExecutionResourceLease {
            authority: Arc::clone(&self.authority.inner),
            _authority: std::marker::PhantomData,
            node: Arc::clone(&self.node),
            policy: self.policy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tests::request, ExecutionWorkCeiling, WorkCeilingDenial};

    #[test]
    fn a_diagnostic_detaches_and_restores_the_invoking_worker() {
        let lease = ExecutionResourceLease::serial(request(1, 65_536, 100)).unwrap();
        let foreign = ExecutionResourceLease::serial(request(1, 65_536, 100)).unwrap();
        let ((), report) = ExecutionWorkCeiling::new(Some(100))
            .run(&lease, || {
                let ((), diagnostic) = ExecutionWorkCeiling::new(Some(100))
                    .run_detached_for_test(&lease, |diagnostic| {
                        ExecutionWorkCeiling::new(Some(100))
                            .run_with_checkpoint(Some(diagnostic), None, |context| {
                                context.checkpoint(3).unwrap();
                            })
                            .unwrap();
                    })
                    .unwrap();
                assert_eq!(diagnostic.charged_work(), 3);
                assert_eq!(
                    foreign.reserve_entry(1).err(),
                    Some(LeaseDenial::UnrelatedNestedLease),
                    "the diagnostic restores the invoking worker's lineage fence"
                );
                assert_eq!(
                    ExecutionWorkCeiling::new(Some(100)).run_detached_for_test(&lease, |_| {
                        panic!("diagnostic panic");
                    }),
                    Err(WorkCeilingDenial::Panicked)
                );
                assert_eq!(
                    foreign.reserve_entry(1).err(),
                    Some(LeaseDenial::UnrelatedNestedLease),
                    "a panicking diagnostic also restores the lineage fence"
                );
            })
            .unwrap();
        assert_eq!(report.charged_work(), 0, "diagnostics own their bill");
        assert_eq!(lease.charged_memory_bytes(), 0);
        assert!(foreign.reserve_entry(1).is_ok());
    }
}
