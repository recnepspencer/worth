use std::ops::Deref;

use worth_proof::NonEmpty;

use super::PhysicalRecordSubmission;

#[derive(Clone)]
pub struct CertificationPhysicalRecordSubmission {
    submission: PhysicalRecordSubmission,
}

impl CertificationPhysicalRecordSubmission {
    pub(super) const fn new(submission: PhysicalRecordSubmission) -> Self {
        Self { submission }
    }

    /// Prepares the classified Store blob route for grouped C5 certification.
    pub fn prepare_blob_record_append(
        &self,
        encoded: Vec<u8>,
        placement: crate::physical_runtime::AdmittedRecordPlacementPolicy,
        request: crate::physical_runtime::PhysicalMutationRequest,
    ) -> crate::physical_runtime::PhysicalMutationPreparationOutcome {
        self.submission
            .prepare_blob_record_append(encoded, placement, request)
    }

    /// Exercises the same selected-root source and destination verifier used
    /// by WAL planning, without granting an append or advancing the root.
    pub fn verify_blob_reuse_claim_before_append(
        &self,
        encoded: Vec<u8>,
        declaration_record: worth_store_physical_format::PersistedRecordIdentity,
        declaration_digest: [u8; 32],
    ) -> Result<(), crate::physical_runtime::RecordAppendDenial> {
        use crate::physical_runtime::record_serving::publication::PreparedReuseDeclarationBasis;
        let director = self
            .submission
            .director
            .upgrade()
            .ok_or(crate::physical_runtime::RecordAppendDenial::PublicationAuthorityReleased)?;
        let batch = crate::physical_runtime::RecordAppendBatch::builder()
            .push_owned(encoded)
            .build()?;
        let generation = director.root_owner.snapshot().0.generation();
        director
            .verify_new_reuse_claim(
                &batch,
                generation,
                PreparedReuseDeclarationBasis {
                    record: declaration_record,
                    digest: declaration_digest,
                },
            )
            .map_err(|error| match error {
                crate::physical_runtime::RecordAppendError::Denied(denial) => denial,
                _ => crate::physical_runtime::RecordAppendDenial::ReuseSourceInvalid,
            })
    }

    pub fn prepare_blob_reuse_claim_append(
        &self,
        encoded: Vec<u8>,
        declaration_record: worth_store_physical_format::PersistedRecordIdentity,
        declaration_digest: [u8; 32],
        placement: crate::physical_runtime::AdmittedRecordPlacementPolicy,
        request: crate::physical_runtime::PhysicalMutationRequest,
    ) -> crate::physical_runtime::PhysicalMutationPreparationOutcome {
        self.submission.prepare_blob_reuse_claim_append(
            encoded,
            declaration_record,
            declaration_digest,
            placement,
            request,
        )
    }

    pub fn cancel_prepared_before_group_seal(
        &self,
        prepared: crate::physical_runtime::PreparedPhysicalMutation,
    ) -> crate::physical_runtime::PhysicalPreSealCancellationOutcome {
        self.submission.cancel_prepared_before_group_seal(prepared)
    }

    pub fn append_prepared_wal_group(
        &self,
        members: NonEmpty<crate::physical_runtime::PreparedPhysicalMutation>,
    ) -> crate::physical_runtime::PhysicalWalGroupAppendOutcome {
        self.submission.append_prepared_wal_group(members)
    }

    pub fn continue_prepared_wal_group(
        &self,
        continuation: crate::physical_runtime::PhysicalWalGroupAppendContinuation,
    ) -> crate::physical_runtime::PhysicalWalGroupAppendOutcome {
        self.submission.continue_prepared_wal_group(continuation)
    }

    pub fn synchronize_appended_wal_group(
        &self,
        appended: crate::physical_runtime::SealedPhysicalDurabilityGroupMembers,
    ) -> crate::physical_runtime::PhysicalWalGroupBarrierOutcome {
        self.submission.synchronize_appended_wal_group(appended)
    }

    pub fn dispatch_wal_durable_data(
        &self,
        durable: crate::physical_runtime::WalDurablePhysicalMutation,
    ) -> crate::physical_runtime::PhysicalDataDispatchOutcome {
        self.submission.dispatch_wal_durable_data(durable)
    }

    pub fn join_data_settled_group(
        &self,
        basis: crate::physical_runtime::PhysicalDurabilityGroupBasis,
        members: NonEmpty<crate::physical_runtime::DataSettledPhysicalMutation>,
    ) -> crate::physical_runtime::PhysicalDataSettledGroupAdmissionOutcome {
        self.submission.join_data_settled_group(basis, members)
    }

    pub fn prepare_root_publication(
        &self,
        settled: crate::physical_runtime::DataSettledPhysicalMutationMembers,
    ) -> crate::physical_runtime::PhysicalRootPublicationPreparationOutcome {
        self.submission.prepare_root_publication(settled)
    }

    pub fn continue_root_publication_preparation(
        &self,
        planning: crate::physical_runtime::RootPublicationPlanningMembers,
    ) -> crate::physical_runtime::PhysicalRootPublicationPreparationOutcome {
        self.submission
            .continue_root_publication_preparation(planning)
    }

    pub fn continue_root_publication_candidate(
        &self,
        candidate: crate::physical_runtime::RootPublicationCandidatePlan,
    ) -> crate::physical_runtime::PhysicalRootPublicationPreparationOutcome {
        self.submission
            .continue_root_publication_candidate(candidate)
    }

    pub fn replace_prepared_root(
        &self,
        prepared: crate::physical_runtime::RootPublicationPreparedPhysicalMutationMembers,
    ) -> crate::physical_runtime::PhysicalRootReplacementOutcome {
        self.submission.replace_prepared_root(prepared)
    }

    pub fn synchronize_replaced_root_namespace(
        &self,
        replaced: crate::physical_runtime::RootReplacedPhysicalMutationMembers,
    ) -> crate::physical_runtime::PhysicalRootNamespaceDurabilityOutcome {
        self.submission
            .synchronize_replaced_root_namespace(replaced)
    }

    pub fn advance_namespace_durable_root(
        &self,
        durable: crate::physical_runtime::RootNamespaceDurablePhysicalMutationMembers,
    ) -> crate::physical_runtime::PhysicalCurrentRootAdvanceOutcome {
        self.submission.advance_namespace_durable_root(durable)
    }
}

impl Deref for CertificationPhysicalRecordSubmission {
    type Target = PhysicalRecordSubmission;

    fn deref(&self) -> &Self::Target {
        &self.submission
    }
}
