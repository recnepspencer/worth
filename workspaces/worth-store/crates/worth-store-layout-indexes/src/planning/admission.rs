use crate::access::shape::{
    access_shapes, AccessLaneClassification, AccessShapeContract, AccessShapeUnsupportedDenial,
};
use crate::materialization::MaterializationDenial;
use crate::planning::AccessPlanSelector;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessPlanningFacade;

impl AccessPlanningFacade {
    pub fn point_access(&self) -> AccessShapeContract {
        access_shapes().point_lookup_declaration()
    }

    pub fn range_access(&self) -> AccessShapeContract {
        access_shapes().range_lookup_declaration()
    }

    pub fn prefix_access(&self) -> AccessShapeContract {
        access_shapes().prefix_lookup_declaration()
    }

    pub fn rebuild_access(
        &self,
        lane: AccessLaneClassification,
    ) -> Result<AccessShapeContract, AccessShapeUnsupportedDenial> {
        access_shapes().rebuild_read_declaration(lane)
    }

    pub fn current_materialization_frontier(
        &self,
        catalog: &crate::BootstrapCatalogReadAdmission,
    ) -> crate::CurrentMaterializationFrontier {
        crate::CurrentMaterializationFrontier::from_catalog(catalog)
    }
    pub fn current_lsm_materialization_frontier(
        &self,
        catalog: &crate::BootstrapCatalogReadAdmission,
        source: &crate::BaselineLsmLookupSource,
    ) -> crate::CurrentMaterializationFrontier {
        crate::CurrentMaterializationFrontier::from_lsm_lookup_source(catalog, source)
    }

    pub fn current_lsm_replay_materialization_frontier(
        self,
        catalog: &crate::BootstrapCatalogReadAdmission,
        source: &worth_store_lsm_authority::AdmittedLsmReplaySource,
    ) -> Result<crate::CurrentMaterializationFrontier, MaterializationDenial> {
        crate::CurrentMaterializationFrontier::from_lsm_replay_source(catalog, source)
    }
    pub fn admit_imported_blob_read_request(
        &self,
        family: crate::AdmittedPhysicalArtifactFamily,
        key_domain: crate::AdmittedPhysicalKeyDomain,
        catalog: &crate::BootstrapCatalogReadAdmission,
        witness: &worth_store_blob_chunks::ImportedBlobWitness,
    ) -> super::ImportedBlobReadAdmissionOutcome {
        super::imported_blob::admit_imported_blob_read_request(family, key_domain, catalog, witness)
    }

    pub const fn selection(&self) -> AccessPlanSelector {
        AccessPlanSelector
    }
}

pub const fn access_planning() -> AccessPlanningFacade {
    AccessPlanningFacade
}
