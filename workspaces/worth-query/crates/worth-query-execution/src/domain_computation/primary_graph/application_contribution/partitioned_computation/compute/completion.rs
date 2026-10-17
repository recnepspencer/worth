//! Value completion preserves the installed collection check before retention handoff.
use super::*;

impl<Schema, Feature, Computation, Owner>
    WorthQueryCompletedPartitionedComputation<Schema, Feature, Computation, Owner>
where
    Schema: ApplicationSchema,
    Feature: ApplicationFeature<Schema>,
    Computation: ApplicationManagedComputation<Schema, Feature>,
    Owner: WorthQueryPartitionedComputationOwner<Schema, Feature, Computation>,
{
    /// All the work the run was charged: digesting the input, deriving and
    /// routing the partition keys, every partition's kernel and every combine,
    /// whether a partition was computed again or carried from the last run.
    pub const fn charged_work(&self) -> u64 {
        self.charged_work
    }

    /// Completes the reduced result on the calling thread.
    pub fn complete(self) -> Result<Owner::Output, Denial<Schema, Feature, Computation, Owner>> {
        if let Some(check) = self.installed.completion {
            let value = self
                .installed
                .owner
                .complete(self.reduced)
                .map_err(WorthQueryPartitionedComputationDenial::Owner)?;
            check(&value).map_err(WorthQueryPartitionedComputationDenial::Owner)?;
            self.deposit.write(self.completed);
            Ok(value)
        } else {
            // Preserve ordinary managed completion's existing retention posture.
            self.deposit.write(self.completed);
            self.installed
                .owner
                .complete(self.reduced)
                .map_err(WorthQueryPartitionedComputationDenial::Owner)
        }
    }
}
