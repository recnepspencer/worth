use super::*;

define_materialization_admission_outcome!(
    BTreePublicationMaterializationAdmissionCase,
    BTreePublicationMaterializationAdmissionOutcome,
    BTreePublicationMaterializationAdmissionView,
    BTreePublicationMaterializationAdmissionCaseId,
    btree_publication_materialization_admission_cases,
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

}
