//! Inline serial admissions share a ledger and never construct a worker pool.
use super::*;

impl ExecutionAuthority {
    /// A ledger for policy-bounded execution on the caller's thread.
    /// It grants one inline worker, creates no threads, and is independent of
    /// the process's sole physical worker-pool authority.
    pub fn serial() -> &'static Self {
        static SERIAL: OnceLock<ExecutionAuthority> = OnceLock::new();
        SERIAL.get_or_init(|| Self {
            inner: Arc::new(AuthorityInner {
                config: ExecutionAuthorityConfig {
                    max_workers: NonZeroUsize::MIN,
                    charged_memory_bytes: None,
                },
                pool: None,
                equivalences: EquivalenceRegistry::new([])
                    .expect("an empty registry has no conflicting contracts"),
                ledger: Mutex::new(Ledger::default()),
            }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ExecutionMap, ExecutionRequest, ExecutionWorkCeiling, MapKernelFailure, MapOutcome,
        MapPartition,
    };
    use worth_foundational::{ExecutionBudget, PartitionIdentity};
    #[test]
    fn an_inline_serial_lease_has_the_same_bill_without_a_worker_pool() {
        let policy = ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, 1 << 20, 100),
        );
        let authority = ExecutionAuthority::serial();
        let wide = ExecutionRequestPolicy::new(
            policy.posture(),
            policy.determinism(),
            ExecutionBudget::new(NonZeroUsize::new(2).unwrap(), 1 << 20, 100),
        );
        assert_eq!(
            authority.admits_policy(&wide),
            Err(super::super::ExecutionPolicyDenial::WorkersExceedAuthority),
            "an inline serial ledger grants no second worker"
        );
        let lease = authority
            .request_lease(LeaseRequest {
                policy,
                deadline: None,
                cancellation: CancellationToken::new(),
            })
            .unwrap();
        assert_eq!(lease.resolved_posture(), ExecutionPosture::Serial);
        let identities = vec![PartitionIdentity::new(1), PartitionIdentity::new(2)];
        let map = ExecutionMap::try_from_declared_partitions(
            identities.clone(),
            identities
                .into_iter()
                .zip([3_u64, 7])
                .map(|(identity, value)| MapPartition {
                    identity,
                    value,
                    read_keys: Vec::<u64>::new(),
                    write_keys: Vec::new(),
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                })
                .collect(),
        )
        .unwrap();
        let (outcome, report) = ExecutionRequest::leased(&lease)
            .run(ExecutionWorkCeiling::new(100), |borrowed| {
                map.run(borrowed, |value, context| {
                    context.checkpoint(*value)?;
                    Ok::<_, MapKernelFailure<()>>(*value)
                })
            })
            .unwrap();
        let MapOutcome::Complete { values, .. } = outcome else {
            panic!("serial work completes")
        };
        assert_eq!(values, [3, 7]);
        assert_eq!(
            (report.charged_work(), report.charged_span()),
            (10, 7),
            "serial placement charges the declared parallel graph's work and critical path"
        );
    }
}
