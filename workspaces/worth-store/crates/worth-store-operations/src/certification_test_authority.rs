use worth_proof::TransitionOutcome;
use worth_store_authority::StoreCurrentAuthorityWitness;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_layout_indexes::access_planning;
use worth_store_layout_indexes::declarations::{
    layout_declarations, AdmittedPhysicalArtifactFamily,
};
use worth_store_layout_indexes::materialization::AdmittedLayoutMaterialization;
use worth_store_security::{
    admit_store_security_scope, StoreCustodyPosture, StoreKeyVersionPosture,
    StoreSecurityScopeAdmissionRequest,
};

use crate::backup::export::current_authority;

pub struct ImportPublicationScenarioPreparation {
    authority: StoreCurrentAuthorityWitness,
    materialization: AdmittedLayoutMaterialization,
}

impl ImportPublicationScenarioPreparation {
    pub const fn authority(&self) -> &StoreCurrentAuthorityWitness {
        &self.authority
    }

    pub fn into_materialization(self) -> AdmittedLayoutMaterialization {
        self.materialization
    }
}

pub fn prepare_import_publication_owner_scenario(
    catalog: &worth_store_layout_indexes::bootstrap::BootstrapCatalogReadAdmission,
) -> ImportPublicationScenarioPreparation {
    let authority = current_authority("store.physical.default_instance");
    let family = admitted_page_family(&authority);
    let materialization = access_planning()
        .admit_current_catalog_root_materialization(family, catalog)
        .into_result()
        .expect("certification publication materialization must admit from the catalog root");
    ImportPublicationScenarioPreparation {
        authority,
        materialization,
    }
}

fn admitted_page_family(
    authority: &StoreCurrentAuthorityWitness,
) -> AdmittedPhysicalArtifactFamily {
    let request = StoreSecurityScopeAdmissionRequest::platform_page_envelope(
        authority,
        StoreKeyVersionPosture::Current,
        StoreCustodyPosture::InternalStoreCustody,
    );
    let scope = match admit_store_security_scope(request) {
        TransitionOutcome::Success(scope) => scope,
        outcome => panic!("publication target security scope must admit: {outcome:?}"),
    };
    let declaration = layout_declarations()
        .declaration(DurableArtifactFamilyId::PhysicalPage)
        .expect("physical page layout family must be declared");
    layout_declarations()
        .admit_physical_artifact_family(declaration, scope.witnesses())
        .unwrap()
}
