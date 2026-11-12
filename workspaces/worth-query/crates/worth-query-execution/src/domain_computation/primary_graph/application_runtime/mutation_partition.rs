//! Checked private partition issuance for application mutation attempts.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;

/// One live issuer, initialized only from its actual Native allocation owner.
pub(in crate::domain_computation::primary_graph) struct ApplicationMutationPartitions {
    next: AtomicU32,
}
impl ApplicationMutationPartitions {
    pub(in crate::domain_computation::primary_graph) fn readmit(
        native: &worth_relational::facade::runtime::RelationalRuntime,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, worth_relational::facade::transactions::RelationalExecutionDenialCause> {
        let highest = native.observe_record_partition_high_water(request)?;
        // MAX is the exhausted sentinel, retaining the prior checked issuance
        // contract even when no successor of the recovered namespace exists.
        let next = highest.map_or(1, |partition| partition.as_u32().saturating_add(1));
        Ok(Self {
            next: AtomicU32::new(next),
        })
    }

    fn issue(&self) -> Option<worth_relational::facade::identity::PartitionId> {
        let mut current = self.next.load(Ordering::Relaxed);
        loop {
            let next = current.checked_add(1)?;
            match self.next.compare_exchange_weak(
                current,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(issued) => return Some(worth_relational::facade::identity::PartitionId(issued)),
                Err(observed) => current = observed,
            }
        }
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn issue_application_mutation_partition(
        &self,
    ) -> Option<worth_relational::facade::identity::PartitionId> {
        self.runtime.mutation_partitions.issue()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_issuance_stops_at_the_exhausted_sentinel() {
        let issuer = ApplicationMutationPartitions {
            next: AtomicU32::new(u32::MAX - 1),
        };
        assert_eq!(issuer.issue().unwrap().as_u32(), u32::MAX - 1);
        assert!(issuer.issue().is_none());
        assert!(issuer.issue().is_none());
        assert_eq!(issuer.next.load(Ordering::Relaxed), u32::MAX);
    }

    #[test]
    fn concurrent_partition_issuance_never_reuses_a_partition() {
        let issuer = std::sync::Arc::new(ApplicationMutationPartitions {
            next: AtomicU32::new(1),
        });
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let issuer = issuer.clone();
                std::thread::spawn(move || {
                    (0..64)
                        .map(|_| issuer.issue().unwrap().as_u32())
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        let mut actual: Vec<_> = workers
            .into_iter()
            .flat_map(|worker| worker.join().unwrap())
            .collect();
        actual.sort_unstable();
        assert_eq!(actual, (1..=512).collect::<Vec<_>>());
    }
}
