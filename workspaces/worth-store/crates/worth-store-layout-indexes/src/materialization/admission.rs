use super::{
    AdmittedCoverageBasis, LayoutCoverageWitness, LayoutMaterializationSourceIdentity,
    LayoutMaterializationState, MaterializationDenial, MaterializationDenialKind,
    PhysicalCoverageBasis,
};
use crate::{AdmittedPhysicalArtifactFamily, BootstrapCatalogReadAdmission};
use worth_store_wal::LogSequenceNumber;

#[macro_use]
mod outcome;
mod admitted_view;
mod btree_admission;
mod catalog_root_admission;
mod import_admission;
mod lsm_admission;

pub use btree_admission::{
    btree_lookup_materialization_admission_cases,
    btree_publication_materialization_admission_cases,
    btree_replay_materialization_admission_cases, BTreeLookupMaterializationAdmissionCaseId,
    BTreeLookupMaterializationAdmissionOutcome, BTreeLookupMaterializationAdmissionView,
    BTreePublicationMaterializationAdmissionCaseId,
    BTreePublicationMaterializationAdmissionOutcome, BTreePublicationMaterializationAdmissionView,
    BTreeReplayMaterializationAdmissionCaseId, BTreeReplayMaterializationAdmissionOutcome,
    BTreeReplayMaterializationAdmissionView,
};
pub use catalog_root_admission::{
    catalog_root_materialization_admission_cases, CatalogRootMaterializationAdmissionCaseId,
    CatalogRootMaterializationAdmissionOutcome, CatalogRootMaterializationAdmissionView,
};
pub use import_admission::{
    imported_blob_materialization_admission_cases, ImportedBlobMaterializationAdmissionCaseId,
    ImportedBlobMaterializationAdmissionOutcome, ImportedBlobMaterializationAdmissionView,
};
pub use lsm_admission::{
    lsm_lookup_materialization_admission_cases, lsm_publication_materialization_admission_cases,
    lsm_replay_materialization_admission_cases, LsmLookupMaterializationAdmissionCaseId,
    LsmLookupMaterializationAdmissionOutcome, LsmLookupMaterializationAdmissionView,
    LsmPublicationMaterializationAdmissionCaseId, LsmPublicationMaterializationAdmissionOutcome,
    LsmPublicationMaterializationAdmissionView, LsmReplayMaterializationAdmissionCaseId,
    LsmReplayMaterializationAdmissionOutcome, LsmReplayMaterializationAdmissionView,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmittedLayoutMaterialization {
    inner: std::sync::Arc<AdmittedLayoutMaterializationData>,
}

#[derive(Debug, PartialEq, Eq)]
struct AdmittedLayoutMaterializationData {
    family: AdmittedPhysicalArtifactFamily,
    coverage: LayoutCoverageWitness,
}

impl AdmittedLayoutMaterialization {
    fn admit_catalog_coverage(
        family: AdmittedPhysicalArtifactFamily,
        catalog: &BootstrapCatalogReadAdmission,
        coverage: LayoutCoverageWitness,
    ) -> Result<Self, MaterializationDenial> {
        if coverage.family() != family.declaration().family() {
            return Err(MaterializationDenial::MaterializationFamilyMismatch);
        }

        let source = LayoutMaterializationSourceIdentity::from_catalog(catalog);
        if coverage.source() != &source {
            return Err(MaterializationDenial::CoverageSourceMismatch);
        }

        Ok(Self::from_admitted_coverage(family, coverage))
    }

    fn admit_current_catalog_root(
        family: AdmittedPhysicalArtifactFamily,
        catalog: &BootstrapCatalogReadAdmission,
    ) -> Result<Self, MaterializationDenial> {
        let source = LayoutMaterializationSourceIdentity::from_catalog(catalog);
        let epoch = worth_store_physical_format::PhysicalEpoch::from_raw(
            catalog.root_owner().generation().get(),
        )
        .expect("admitted catalog root generation is nonzero");
        let basis = PhysicalCoverageBasis::root_epoch(epoch);
        let admitted_basis = AdmittedCoverageBasis::admit(source.clone(), &basis);
        let state =
            LayoutMaterializationState::exact_through_physical_basis(family.declaration().family());
        let coverage = LayoutCoverageWitness::from_admitted_bases(
            state,
            admitted_basis.clone(),
            admitted_basis,
            None,
        )?;
        Self::admit_catalog_coverage(family, catalog, coverage)
    }

    fn admit_btree_publication_exact(
        family: AdmittedPhysicalArtifactFamily,
        catalog: &BootstrapCatalogReadAdmission,
        publication: worth_store_physical_format::RootPublicationValidationWitness,
    ) -> Result<Self, MaterializationDenial> {
        let root = publication.reference();
        let source =
            LayoutMaterializationSourceIdentity::from_btree_publication(catalog, publication);
        let epoch = worth_store_physical_format::PhysicalEpoch::from_raw(root.generation().get())
            .expect("admitted physical root generation is nonzero");
        Self::admit_exact_from_source(
            family,
            catalog,
            source,
            PhysicalCoverageBasis::root_epoch(epoch),
        )
    }

    fn admit_btree_lookup_exact(
        family: AdmittedPhysicalArtifactFamily,
        catalog: &BootstrapCatalogReadAdmission,
        source: &crate::BaselineBTreeReadSource,
    ) -> Result<Self, MaterializationDenial> {
        if source.store_authority_identity() != family.authority_identity() {
            return Err(MaterializationDenial::BTreeSourceStoreAuthorityMismatch);
        }
        let identity =
            LayoutMaterializationSourceIdentity::from_btree_lookup_source(catalog, source);
        let epoch = worth_store_physical_format::PhysicalEpoch::from_raw(
            source.root_reference().generation().get(),
        )
        .expect("admitted physical root generation is nonzero");
        Self::admit_exact_from_source(
            family,
            catalog,
            identity,
            PhysicalCoverageBasis::root_epoch(epoch),
        )
    }

    fn admit_btree_replay_exact(
        family: AdmittedPhysicalArtifactFamily,
        catalog: &BootstrapCatalogReadAdmission,
        source: &crate::AdmittedBTreeReplayPhysicalSource,
    ) -> Result<Self, MaterializationDenial> {
        let identity =
            LayoutMaterializationSourceIdentity::from_btree_replay_source(catalog, source);
        let epoch = worth_store_physical_format::PhysicalEpoch::from_raw(
            source.root_reference().generation().get(),
        )
        .expect("admitted replay root generation is nonzero");
        Self::admit_exact_from_source(
            family,
            catalog,
            identity,
            PhysicalCoverageBasis::root_epoch(epoch),
        )
    }

    fn admit_lsm_lookup_exact(
        family: AdmittedPhysicalArtifactFamily,
        catalog: &BootstrapCatalogReadAdmission,
        source: &crate::strategy::BaselineLsmLookupSource,
    ) -> Result<Self, MaterializationDenial> {
        Self::require_lsm_source_authority(family, source.publication().key())?;
        Self::admit_lsm_replacement_exact(
            family,
            catalog,
            LayoutMaterializationSourceIdentity::from_lsm_lookup_source(catalog, source),
            source.replacement_output(),
        )
    }

    fn admit_lsm_publication_exact(
        family: AdmittedPhysicalArtifactFamily,
        catalog: &BootstrapCatalogReadAdmission,
        execution: &crate::strategy::BaselineLsmManifestPublicationExecution,
    ) -> Result<Self, MaterializationDenial> {
        let publication = execution.membership_replacement();
        Self::require_lsm_source_authority(family, publication.key())?;
        Self::admit_lsm_replacement_exact(
            family,
            catalog,
            LayoutMaterializationSourceIdentity::from_lsm_publication(catalog, publication),
            publication.output(),
        )
    }

    fn admit_lsm_replacement_exact(
        family: AdmittedPhysicalArtifactFamily,
        catalog: &BootstrapCatalogReadAdmission,
        source: LayoutMaterializationSourceIdentity,
        replacement: worth_store_wal::BlobWalRecordIdentity,
    ) -> Result<Self, MaterializationDenial> {
        Self::admit_exact_from_source(
            family,
            catalog,
            source,
            PhysicalCoverageBasis::wal_lsn(LogSequenceNumber::new(replacement.sequence())),
        )
    }

    fn admit_lsm_replay_exact(
        family: AdmittedPhysicalArtifactFamily,
        catalog: &BootstrapCatalogReadAdmission,
        source: &worth_store_lsm_authority::AdmittedLsmReplaySource,
    ) -> Result<Self, MaterializationDenial> {
        Self::require_lsm_source_authority(family, source.membership().key())?;
        let (_, covered_lsn) = source.selected_lsn_range();
        let identity =
            LayoutMaterializationSourceIdentity::from_lsm_replay_source(catalog, source)?;
        Self::admit_exact_from_source(
            family,
            catalog,
            identity,
            PhysicalCoverageBasis::wal_lsn(LogSequenceNumber::new(covered_lsn)),
        )
    }

    fn require_lsm_source_authority(
        family: AdmittedPhysicalArtifactFamily,
        key: worth_store_lsm_authority::LsmMembershipKey,
    ) -> Result<(), MaterializationDenial> {
        if key.authority_identity() != family.authority_identity() {
            return Err(MaterializationDenial::LsmSourceStoreAuthorityMismatch);
        }
        if key.security_identity() != family.security_identity() {
            return Err(MaterializationDenial::LsmSourceSecurityScopeMismatch);
        }
        Ok(())
    }

    fn admit_imported_blob_exact(
        family: AdmittedPhysicalArtifactFamily,
        catalog: &BootstrapCatalogReadAdmission,
        witness: &worth_store_blob_chunks::ImportedBlobWitness,
    ) -> Result<Self, MaterializationDenial> {
        if family.family_id() != worth_store_contracts::DurableArtifactFamilyId::BlobManifest {
            return Err(MaterializationDenial::ImportedBlobFamilyRequired);
        }
        if family.security_identity() != witness.security_metadata().identity() {
            return Err(MaterializationDenial::ImportedBlobSecurityScopeMismatch);
        }
        if family.authority_identity() != witness.authority_identity() {
            return Err(MaterializationDenial::ImportedBlobStoreAuthorityMismatch);
        }

        Self::admit_exact_from_source(
            family,
            catalog,
            LayoutMaterializationSourceIdentity::from_imported_blob(catalog, witness),
            PhysicalCoverageBasis::blob_generation(crate::BlobGenerationBasis::from_sequence(
                witness.generation().sequence(),
            )),
        )
    }

    fn admit_exact_from_source(
        family: AdmittedPhysicalArtifactFamily,
        _catalog: &BootstrapCatalogReadAdmission,
        source: LayoutMaterializationSourceIdentity,
        basis: PhysicalCoverageBasis,
    ) -> Result<Self, MaterializationDenial> {
        let admitted_basis = AdmittedCoverageBasis::admit(source.clone(), &basis);
        let state =
            LayoutMaterializationState::exact_through_physical_basis(family.declaration().family());
        let coverage = LayoutCoverageWitness::from_admitted_bases(
            state,
            admitted_basis.clone(),
            admitted_basis,
            None,
        )?;
        Ok(Self::from_admitted_coverage(family, coverage))
    }
}
