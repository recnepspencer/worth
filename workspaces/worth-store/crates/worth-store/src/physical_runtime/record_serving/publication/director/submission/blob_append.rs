use worth_proof::TransitionOutcome;

use super::PhysicalRecordSubmission;
use crate::physical_runtime::{
    record_serving::{
        publication::{PhysicalMutationPreparationOutcome, PhysicalMutationPreparationStale},
        AdmittedRecordPlacementPolicy,
    },
    PhysicalMutationRequest,
};

impl PhysicalRecordSubmission {
    pub(in crate::physical_runtime) fn prepare_blob_record_append(
        &self,
        encoded: Vec<u8>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into();
        };
        director.prepare_blob_record_append(encoded, placement, request)
    }

    pub(in crate::physical_runtime) fn prepare_blob_reuse_claim_append(
        &self,
        encoded: Vec<u8>,
        declaration_record: worth_store_physical_format::PersistedRecordIdentity,
        declaration_digest: [u8; 32],
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into();
        };
        director.prepare_blob_reuse_claim_append(
            encoded,
            declaration_record,
            declaration_digest,
            placement,
            request,
        )
    }

    pub(in crate::physical_runtime) fn prepare_blob_dedupe_quarantine_append(
        &self,
        encoded: Vec<u8>,
        declaration_record: worth_store_physical_format::PersistedRecordIdentity,
        declaration_digest: [u8; 32],
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into();
        };
        director.prepare_blob_dedupe_quarantine_append(
            encoded,
            declaration_record,
            declaration_digest,
            placement,
            request,
        )
    }

    pub(in crate::physical_runtime) fn prepare_blob_derived_directory_append(
        &self,
        encoded: Vec<u8>,
        expected_previous: Option<worth_store_physical_format::DerivedFamilyRootDirectoryBinding>,
        replaced_nodes: crate::physical_runtime::layout::AdmittedDirectoryRetirement<'_>,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into();
        };
        director.prepare_blob_derived_directory_append(
            encoded,
            expected_previous,
            replaced_nodes,
            placement,
            request,
        )
    }

    pub(in crate::physical_runtime) fn prepare_btree_node_append(
        &self,
        encoded: Vec<u8>,
        expected_family: worth_store_contracts::DurableArtifactFamilyId,
        placement: AdmittedRecordPlacementPolicy,
        request: PhysicalMutationRequest,
    ) -> PhysicalMutationPreparationOutcome {
        let Some(director) = self.director.upgrade() else {
            return TransitionOutcome::stale(
                PhysicalMutationPreparationStale::PublicationAuthorityReleased,
            )
            .into();
        };
        director.prepare_btree_node_append(encoded, expected_family, placement, request)
    }
}
