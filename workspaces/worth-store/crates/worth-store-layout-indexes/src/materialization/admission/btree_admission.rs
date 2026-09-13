use super::*;

define_materialization_admission_outcome!(
    BTreePublicationMaterializationAdmissionCase,
    BTreePublicationMaterializationAdmissionOutcome,
    BTreePublicationMaterializationAdmissionView,
    BTreePublicationMaterializationAdmissionCaseId,
    btree_publication_materialization_admission_cases,
    []
);
define_materialization_admission_outcome!(
    BTreeLookupMaterializationAdmissionCase,
    BTreeLookupMaterializationAdmissionOutcome,
    BTreeLookupMaterializationAdmissionView,
    BTreeLookupMaterializationAdmissionCaseId,
    btree_lookup_materialization_admission_cases,
    [MaterializationDenialKind::BTreeSourceStoreAuthorityMismatch]
);
define_materialization_admission_outcome!(
    BTreeReplayMaterializationAdmissionCase,
    BTreeReplayMaterializationAdmissionOutcome,
    BTreeReplayMaterializationAdmissionView,
    BTreeReplayMaterializationAdmissionCaseId,
    btree_replay_materialization_admission_cases,
    []
);

impl crate::planning::AccessPlanningFacade {
    pub fn admit_btree_publication_materialization(
        &self,
        family: crate::AdmittedPhysicalArtifactFamily,
        catalog: &crate::BootstrapCatalogReadAdmission,
        publication: worth_store_physical_format::RootPublicationValidationWitness,
    ) -> BTreePublicationMaterializationAdmissionOutcome {
        BTreePublicationMaterializationAdmissionOutcome::issue(
            AdmittedLayoutMaterialization::admit_btree_publication_exact(
                family,
                catalog,
                publication,
            ),
        )
    }

    pub fn admit_btree_lookup_materialization(
        &self,
        family: crate::AdmittedPhysicalArtifactFamily,
        catalog: &crate::BootstrapCatalogReadAdmission,
        source: &crate::BaselineBTreeReadSource,
    ) -> BTreeLookupMaterializationAdmissionOutcome {
        BTreeLookupMaterializationAdmissionOutcome::issue(
            AdmittedLayoutMaterialization::admit_btree_lookup_exact(family, catalog, source),
        )
    }

    pub fn admit_btree_replay_materialization(
        &self,
        family: crate::AdmittedPhysicalArtifactFamily,
        catalog: &crate::BootstrapCatalogReadAdmission,
        source: &crate::AdmittedBTreeReplayPhysicalSource,
    ) -> BTreeReplayMaterializationAdmissionOutcome {
        BTreeReplayMaterializationAdmissionOutcome::issue(
            AdmittedLayoutMaterialization::admit_btree_replay_exact(family, catalog, source),
        )
    }
}
