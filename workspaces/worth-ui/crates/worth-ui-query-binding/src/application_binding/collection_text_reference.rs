use worth_query::facade::{foundation::ObservationLaneWitness, installed, runtime};

use crate::{
    installed_domain::collection_text_projection::{
        WorthUiCollectionTextProjection, WorthUiCollectionTextProjectionFamily,
    },
    WorthUiInstalledQueryDomain, WorthUiQueryHost, WorthUiQueryOperationAttemptDenial,
};

pub(crate) type WorthUiBoundCollectionTextProjection<L> = installed::WorthQueryBoundDomainOperation<
    crate::WorthUiDomainEntry,
    WorthUiCollectionTextProjection,
    WorthUiCollectionTextProjectionFamily,
    L,
>;

#[derive(Clone)]
pub(crate) struct WorthUiInstalledCollectionTextOperationReference {
    installed_domain: WorthUiInstalledQueryDomain,
}

pub(crate) struct WorthUiCollectionTextOperatingWorldGateway<'runtime> {
    world: installed::WorthQueryInstalledOperatingWorld<'runtime, ObservationLaneWitness>,
    reference: WorthUiInstalledCollectionTextOperationReference,
}

impl WorthUiInstalledQueryDomain {
    pub(crate) fn collection_text_operation_reference(
        &self,
    ) -> WorthUiInstalledCollectionTextOperationReference {
        WorthUiInstalledCollectionTextOperationReference {
            installed_domain: self.clone(),
        }
    }
}

impl WorthUiInstalledCollectionTextOperationReference {
    pub(crate) fn enter_attempt<'runtime>(
        &self,
        workspace: &'runtime runtime::WorthQueryWorkspace,
    ) -> Result<
        WorthUiCollectionTextOperatingWorldGateway<'runtime>,
        WorthUiQueryOperationAttemptDenial,
    > {
        let current = WorthUiQueryHost::from_workspace(workspace)
            .installed_domain()
            .map_err(WorthUiQueryOperationAttemptDenial::Installation)?;
        if !current.shares_authority_with(&self.installed_domain) {
            return Err(WorthUiQueryOperationAttemptDenial::InstalledDomainAuthorityMismatch);
        }
        Ok(WorthUiCollectionTextOperatingWorldGateway {
            world: workspace
                .observe_operating_world(workspace.current_world())
                .map_err(|denial| {
                    WorthUiQueryOperationAttemptDenial::OperatingWorld(Box::new(denial))
                })?,
            reference: self.clone(),
        })
    }

    pub(crate) fn installation_is_current(&self) -> bool {
        self.installed_domain.handle().installation_is_current()
    }
}

impl WorthUiCollectionTextOperatingWorldGateway<'_> {
    pub(crate) fn bind(
        self,
    ) -> Result<
        (
            WorthUiInstalledCollectionTextOperationReference,
            WorthUiBoundCollectionTextProjection<ObservationLaneWitness>,
        ),
        Box<installed::WorthQueryOperationBindingDenial>,
    > {
        let bound = self
            .world
            .family(WorthUiCollectionTextProjectionFamily)
            .bind(
                self.reference.installed_domain.handle(),
                WorthUiCollectionTextProjection,
            )
            .map_err(Box::new)?;
        Ok((self.reference, bound))
    }
}
