use worth_query::facade::{foundation::ObservationLaneWitness, installed, runtime};

use crate::{
    WorthUiInstalledQueryDomain, WorthUiQueryHost, WorthUiQueryOperationAttemptDenial,
    WorthUiScalarTextProjection, WorthUiScalarTextProjectionFamily,
};

pub(crate) type WorthUiBoundScalarTextProjection<L> = installed::WorthQueryBoundDomainOperation<
    crate::WorthUiDomainEntry,
    WorthUiScalarTextProjection,
    WorthUiScalarTextProjectionFamily,
    L,
>;

#[derive(Clone)]
pub(crate) struct WorthUiInstalledScalarTextOperationReference {
    installed_domain: WorthUiInstalledQueryDomain,
}

pub(crate) struct WorthUiScalarTextOperatingWorldGateway<'runtime> {
    world: installed::WorthQueryInstalledOperatingWorld<'runtime, ObservationLaneWitness>,
    reference: WorthUiInstalledScalarTextOperationReference,
}

impl WorthUiInstalledQueryDomain {
    pub(crate) fn scalar_text_operation_reference(
        &self,
    ) -> WorthUiInstalledScalarTextOperationReference {
        WorthUiInstalledScalarTextOperationReference {
            installed_domain: self.clone(),
        }
    }
}

impl WorthUiInstalledScalarTextOperationReference {
    pub(crate) fn enter_attempt<'runtime>(
        &self,
        workspace: &'runtime runtime::WorthQueryWorkspace,
    ) -> Result<WorthUiScalarTextOperatingWorldGateway<'runtime>, WorthUiQueryOperationAttemptDenial>
    {
        let current = WorthUiQueryHost::from_workspace(workspace)
            .installed_domain()
            .map_err(WorthUiQueryOperationAttemptDenial::Installation)?;
        if !current.shares_authority_with(&self.installed_domain) {
            return Err(WorthUiQueryOperationAttemptDenial::InstalledDomainAuthorityMismatch);
        }
        Ok(WorthUiScalarTextOperatingWorldGateway {
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

impl WorthUiScalarTextOperatingWorldGateway<'_> {
    pub(crate) fn bind(
        self,
    ) -> Result<
        (
            WorthUiInstalledScalarTextOperationReference,
            WorthUiBoundScalarTextProjection<ObservationLaneWitness>,
        ),
        Box<installed::WorthQueryOperationBindingDenial>,
    > {
        let bound = self
            .world
            .family(WorthUiScalarTextProjectionFamily)
            .bind(
                self.reference.installed_domain.handle(),
                WorthUiScalarTextProjection,
            )
            .map_err(Box::new)?;
        Ok((self.reference, bound))
    }
}
