use std::collections::BTreeMap;
use std::num::NonZeroU64;

use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, CheckpointSelectiveRecordAggregate,
    PhysicalCheckpointIdentity, PhysicalCheckpointSource,
};
use worth_store_physical_integrity::{
    IntegrityValidatedCheckpointBinding, UntrustedPhysicalArtifact, VerifiedCheckpointStream,
};

use crate::physical_runtime::durability::{
    DecodedPhysicalMutationBindingRecord, PhysicalBindingDecodingContext,
};
use crate::physical_runtime::{PhysicalDurabilityPolicyIdentity, PhysicalIdempotencyPolicy};

use super::{
    checkpoint_evidence, empty_failure, merge_evidence, sample_failure,
    StoreRecoveryBindingSampleDenial, StoreRecoveryBindingSampleFailure,
    StoreRecoveryOperationEvidence,
};

/// Store-owned semantic checkpoint binding basis. Recovery carries this beside
/// the verified checkpoint; physical source selection never observes it.
pub struct StoreRecoveryCheckpointBindingBasis {
    checkpoint: PhysicalCheckpointIdentity,
    compaction_generation: u64,
    binding_count: u64,
    binding_bytes: u64,
    binding_digest: [u8; 32],
    outcome: Result<Box<[StoreRecoveryOperationEvidence]>, StoreRecoveryBindingSampleFailure>,
}

/// Bounded, uncommitted Store interpretation of integrity-admitted bindings.
/// Only `finish` can produce a reusable basis, and it requires the aggregate-
/// admitted checkpoint owner projection.
pub struct StoreRecoveryCheckpointBindingRebuilder {
    checkpoint: PhysicalCheckpointIdentity,
    compaction_generation: u64,
    maximum_operations: u64,
    aggregate: CheckpointSelectiveRecordAggregate,
    context: Option<PhysicalBindingDecodingContext>,
    operations: BTreeMap<[u8; 32], StoreRecoveryOperationEvidence>,
    failure: Option<StoreRecoveryBindingSampleFailure>,
}

impl StoreRecoveryCheckpointBindingRebuilder {
    pub fn begin(
        store: StableStoreIdentity,
        source: PhysicalCheckpointSource,
        compaction_generation: u64,
        maximum_operations: u64,
    ) -> Self {
        let mut failure = None;
        let context = if source.identity().store_identity() != store {
            failure = Some(empty_failure(
                StoreRecoveryBindingSampleDenial::ForeignCheckpoint,
            ));
            None
        } else if let Some(security) = source.security_binding() {
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
        Self {
            checkpoint: source.identity(),
            compaction_generation,
            maximum_operations,
            aggregate: CheckpointSelectiveRecordAggregate::new(),
            context,
            operations: BTreeMap::new(),
            failure,
        }
    }

    pub fn consume(
        &mut self,
        admitted: &IntegrityValidatedCheckpointBinding<'_>,
        exact_record: UntrustedPhysicalArtifact<'_>,
    ) {
        if self.aggregate.include(exact_record.bytes()).is_err() {
            self.reject(StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding);
            return;
        }
        let Some(context) = self.context else {
            return;
        };
        if self.failure.is_some() {
            return;
        }
        let decoded = admitted
            .project_payload(exact_record, self.checkpoint)
            .ok()
            .and_then(|projection| {
                DecodedPhysicalMutationBindingRecord::decode(
                    &exact_record.bytes()[projection.payload_range()],
                    context,
                )
                .ok()
            });
        let Some(decoded) = decoded else {
            self.reject(StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding);
            return;
        };
        let evidence = checkpoint_evidence(decoded, self.compaction_generation);
        if let Err(denial) = merge_evidence(&mut self.operations, evidence, self.maximum_operations)
        {
            self.reject(denial);
        }
    }

    pub fn finish(
        self,
        checkpoint: &VerifiedCheckpointStream,
    ) -> StoreRecoveryCheckpointBindingBasis {
        let footer = checkpoint.footer();
        let summary = self.aggregate.summary();
        let aggregate_matches = checkpoint.source().identity() == self.checkpoint
            && checkpoint.compaction_cutover().product_generation() == self.compaction_generation
            && footer.binding_record_count() == summary.record_count()
            && footer.binding_record_bytes() == summary.encoded_bytes()
            && footer.binding_records_digest() == summary.digest();
        let outcome = if !aggregate_matches {
            Err(sample_failure(
                StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding,
                &self.operations,
                0,
                0,
            ))
        } else if let Some(failure) = self.failure {
            Err(failure)
        } else {
            Ok(self
                .operations
                .into_values()
                .collect::<Vec<_>>()
                .into_boxed_slice())
        };
        StoreRecoveryCheckpointBindingBasis {
            checkpoint: self.checkpoint,
            compaction_generation: self.compaction_generation,
            binding_count: summary.record_count(),
            binding_bytes: summary.encoded_bytes(),
            binding_digest: summary.digest(),
            outcome,
        }
    }

    fn reject(&mut self, denial: StoreRecoveryBindingSampleDenial) {
        if self.failure.is_none() {
            self.failure = Some(sample_failure(denial, &self.operations, 0, 0));
        }
    }
}

impl StoreRecoveryCheckpointBindingBasis {
    pub(super) fn operations(
        &self,
        checkpoint: &VerifiedCheckpointStream,
        maximum: u64,
    ) -> Result<BTreeMap<[u8; 32], StoreRecoveryOperationEvidence>, StoreRecoveryBindingSampleFailure>
    {
        let footer = checkpoint.footer();
        if checkpoint.source().identity() != self.checkpoint
            || checkpoint.compaction_cutover().product_generation() != self.compaction_generation
            || footer.binding_record_count() != self.binding_count
            || footer.binding_record_bytes() != self.binding_bytes
            || footer.binding_records_digest() != self.binding_digest
        {
            return Err(empty_failure(
                StoreRecoveryBindingSampleDenial::InvalidCheckpointBinding,
            ));
        }
        let evidence = self.outcome.as_ref().map_err(|failure| *failure)?;
        if evidence.len() as u64 > maximum {
            let operations = evidence
                .iter()
                .cloned()
                .map(|item| (item.idempotency_identity, item))
                .collect();
            return Err(sample_failure(
                StoreRecoveryBindingSampleDenial::OperationBindingLimit,
                &operations,
                0,
                0,
            ));
        }
        Ok(evidence
            .iter()
            .cloned()
            .map(|item| (item.idempotency_identity, item))
            .collect())
    }
}
