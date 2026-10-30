//! Checked private partition issuance for application mutation attempts.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use std::sync::atomic::Ordering;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn issue_application_mutation_partition(
        &self,
    ) -> Option<worth_relational::facade::identity::PartitionId> {
        {
            let update_counter = &self.next_application_mutation_partition;
            let mut current = update_counter.load(Ordering::Relaxed);
            loop {
                let Some(updated_value) = current.checked_add(1) else {
                    break Err(current);
                };
                match update_counter.compare_exchange_weak(
                    current,
                    updated_value,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => current = reloaded_value,
                }
            }
        }
        .ok()
        .map(worth_relational::facade::identity::PartitionId)
    }
}
