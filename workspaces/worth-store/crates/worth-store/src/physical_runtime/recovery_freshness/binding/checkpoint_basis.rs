use std::num::NonZeroU64;

use worth_store_physical_format::CheckpointSelectiveRecordAggregate;
use worth_store_physical_integrity::{
    IntegrityValidatedCheckpointBinding, UntrustedPhysicalArtifact, VerifiedCheckpointFacts,
};

use crate::physical_runtime::durability::PhysicalBindingDecodingContext;
use crate::physical_runtime::{
    PhysicalDurabilityPolicyIdentity, PhysicalIdempotencyPolicy, PhysicalRecoveryReadAllocation,
    SharedRecoveryCheckpoint,
};

mod backing;
mod decode;
use super::operations::{RecoveryBindingOperations, RecoveryBindingOperationsCapacity};
use backing::CheckpointBindingBacking;
pub use decode::StoreRecoveryCheckpointBindingAllocationDenial;

#[cfg(all(test, feature = "certification-test-authority"))]
mod tests;
#[cfg(test)]
pub(in crate::physical_runtime) use decode::checkpoint_binding_decode_peak;
pub(in crate::physical_runtime) use decode::decode_checkpoint_evidence;

use super::{
    empty_failure, sample_failure_from_evidence, StoreRecoveryBindingSampleDenial,
    StoreRecoveryBindingSampleFailure, StoreRecoveryOperationEvidence,
};

/// Store-owned semantic checkpoint binding basis. Recovery carries this beside
/// the verified checkpoint; physical source selection never observes it.
pub struct StoreRecoveryCheckpointBindingBasis {
    facts: VerifiedCheckpointFacts,
    // Evidence drops before its required native reservation.
    outcome: Result<Vec<StoreRecoveryOperationEvidence>, StoreRecoveryBindingSampleFailure>,
    backing: CheckpointBindingBacking,
}

impl StoreRecoveryCheckpointBindingBasis {
    /// The reusable basis retains operation evidence, not checkpoint stream bytes.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        match &self.outcome {
            Ok(operations) => (operations.capacity() as u64)
                .checked_mul(std::mem::size_of::<StoreRecoveryOperationEvidence>() as u64),
            Err(_) => Some(0),
        }
    }

    pub fn charged_bytes(&self) -> u64 {
        self.backing.charged_bytes()
    }

    pub(in crate::physical_runtime) fn matches_owner(
        &self,
        owner: &crate::physical_runtime::instance::PhysicalResidencyOwner,
    ) -> bool {
        self.facts.source().identity().store_identity() == owner.ports().store_identity()
            && self.backing.matches_owner(owner)
    }
}

/// Bounded, uncommitted Store interpretation of integrity-admitted bindings.
/// Only `finish` can produce a reusable basis, and it requires the aggregate-
/// admitted checkpoint owner projection.
pub struct StoreRecoveryCheckpointBindingRebuilder {
    facts: VerifiedCheckpointFacts,
    aggregate: CheckpointSelectiveRecordAggregate,
    context: Option<PhysicalBindingDecodingContext>,
    operations: RecoveryBindingOperations,
    failure: Option<StoreRecoveryBindingSampleFailure>,
    backing: CheckpointBindingBacking,
}

impl StoreRecoveryCheckpointBindingRebuilder {
    pub(in crate::physical_runtime) fn prepare(
        allocation: &mut PhysicalRecoveryReadAllocation<'_>,
        checkpoint: &SharedRecoveryCheckpoint,
        maximum_operations: u64,
    ) -> Result<Self, StoreRecoveryCheckpointBindingAllocationDenial> {
        use StoreRecoveryCheckpointBindingAllocationDenial as Denial;
        let facts = checkpoint.facts();
        let source = facts.source();
        let store = allocation.store_identity();
        if source.identity().store_identity() != store {
            return Err(Denial::StoreMismatch);
        }
        if !allocation.owns_checkpoint(checkpoint) {
            return Err(Denial::PoolMismatch);
        }
        let mut failure = None;
        let context = if let Some(security) = source.security_binding() {
            if let Some(retention) = NonZeroU64::new(security.idempotency_retention_generations()) {
                let policy = PhysicalDurabilityPolicyIdentity::from_recovery_binding(
                    security.policy_identity(),
                );
                let idempotency = PhysicalIdempotencyPolicy::from_recovery_binding(retention);
                Some(PhysicalBindingDecodingContext::new(
                    store,
                    policy,
                    idempotency,
                ))
            } else {
                failure = Some(empty_failure(
                    StoreRecoveryBindingSampleDenial::InvalidCheckpointSecurityBinding,
                ));
                None
            }
        } else {
            failure = Some(empty_failure(
                StoreRecoveryBindingSampleDenial::MissingCheckpointSecurityBinding,
            ));
            None
        };
        let capacity = RecoveryBindingOperationsCapacity::for_records(if failure.is_some() {
            0
        } else {
            facts.footer().binding_record_count()
        })?;
        // Declare backing first: failed preparation drops partial Vecs before
        // this grant, and the returned owner declares its storage before backing.
        let backing = CheckpointBindingBacking::admit(allocation, capacity.requested_bytes())?;
        let operations = match backing.grant() {
            Some(grant) => RecoveryBindingOperations::prepare(capacity, maximum_operations, grant)?,
            None => RecoveryBindingOperations::empty(maximum_operations),
        };
        Ok(Self {
            facts,
            aggregate: CheckpointSelectiveRecordAggregate::new(),
            context,
            operations,
            failure,
            backing,
        })
    }

    pub fn consume(
        &mut self,
        admitted: &IntegrityValidatedCheckpointBinding<'_>,
        exact_record: UntrustedPhysicalArtifact<'_>,
        allocation: &mut PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<(), StoreRecoveryCheckpointBindingAllocationDenial> {
        if allocation.store_identity() != self.facts.source().identity().store_identity() {
            self.reject(StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding);
            return Err(StoreRecoveryCheckpointBindingAllocationDenial::StoreMismatch);
        }
        if !self.backing.matches_window(allocation) {
            self.reject(StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding);
            return Err(StoreRecoveryCheckpointBindingAllocationDenial::PoolMismatch);
        }
        if self.aggregate.include(exact_record.bytes()).is_err() {
            self.reject(StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding);
            return Ok(());
        }
        if self.aggregate.summary().record_count() > self.facts.footer().binding_record_count() {
            self.reject(StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding);
            return Ok(());
        }
        let Some(context) = self.context else {
            return Ok(());
        };
        if self.failure.is_some() {
            return Ok(());
        }
        let Ok(projection) = admitted.project_payload(exact_record, self.facts.source().identity())
        else {
            self.reject(StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding);
            return Ok(());
        };
        let decoded = decode_checkpoint_evidence(
            allocation,
            &exact_record.bytes()[projection.payload_range()],
            context,
            self.facts.compaction_cutover().product_generation(),
        );
        let evidence = match decoded {
            Ok(evidence) => evidence,
            Err(denial) => {
                self.reject(StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding);
                return Err(denial);
            }
        };
        let Some(evidence) = evidence else {
            self.reject(StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding);
            return Ok(());
        };
        if let Err(denial) = self.operations.merge(evidence) {
            self.reject(denial);
        }
        Ok(())
    }

    pub fn finish(
        self,
    ) -> Result<StoreRecoveryCheckpointBindingBasis, StoreRecoveryCheckpointBindingAllocationDenial>
    {
        let footer = self.facts.footer();
        let summary = self.aggregate.summary();
        let aggregate_matches = footer.binding_record_count() == summary.record_count()
            && footer.binding_record_bytes() == summary.encoded_bytes()
            && footer.binding_records_digest() == summary.digest();
        let mut backing = self.backing;
        let outcome = if !aggregate_matches {
            let failure = sample_failure_from_evidence(
                StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding,
                self.operations.evidence().iter(),
                0,
                0,
            );
            drop(self.operations);
            Err(failure)
        } else if let Some(failure) = self.failure {
            drop(self.operations);
            Err(failure)
        } else {
            Ok(self.operations.into_evidence())
        };
        let retained = match &outcome {
            Ok(evidence) => (evidence.capacity() as u64)
                .checked_mul(std::mem::size_of::<StoreRecoveryOperationEvidence>() as u64)
                .ok_or(StoreRecoveryCheckpointBindingAllocationDenial::SizeOverflow)?,
            Err(_) => 0,
        };
        backing.retain(retained)?;
        Ok(StoreRecoveryCheckpointBindingBasis {
            facts: self.facts,
            outcome,
            backing,
        })
    }

    fn reject(&mut self, denial: StoreRecoveryBindingSampleDenial) {
        if self.failure.is_none() {
            self.failure = Some(sample_failure_from_evidence(
                denial,
                self.operations.evidence().iter(),
                0,
                0,
            ));
        }
    }
}

impl StoreRecoveryCheckpointBindingBasis {
    pub(in crate::physical_runtime) fn matches_checkpoint(
        &self,
        checkpoint: &VerifiedCheckpointFacts,
    ) -> bool {
        self.facts == *checkpoint
    }

    pub(super) fn evidence(
        &self,
        checkpoint: &VerifiedCheckpointFacts,
        maximum: u64,
    ) -> Result<&[StoreRecoveryOperationEvidence], StoreRecoveryBindingSampleFailure> {
        if !self.matches_checkpoint(checkpoint) {
            return Err(empty_failure(
                StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding,
            ));
        }
        let evidence = self.outcome.as_ref().map_err(Clone::clone)?;
        if evidence.len() as u64 > maximum {
            return Err(sample_failure_from_evidence(
                StoreRecoveryBindingSampleDenial::OperationBindingLimit,
                evidence.iter(),
                0,
                0,
            ));
        }
        Ok(evidence)
    }
}
