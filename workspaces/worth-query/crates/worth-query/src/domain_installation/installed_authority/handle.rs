use std::marker::PhantomData;
use std::sync::Arc;

use super::{
    WorthQueryDomainGraphReadOperationDefinition, WorthQueryDomainHandleDenial,
    WorthQueryDomainInstallationGeneration, WorthQueryDomainPackageIdentity,
    WorthQueryDomainRebindRequest, WorthQueryInstalledDomainAuthority,
    WorthQueryInstalledDomainAuthorityWitness, WorthQueryInstalledDomainDeclarationContext,
    WorthQueryInstalledDomainDeclarationContextDenial, WorthQueryInstalledGraphReadOperation,
};

#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryInstalledDomainHandle<D> {
    pub(crate) authority: Arc<WorthQueryInstalledDomainAuthority>,
    marker: PhantomData<fn() -> D>,
}

impl<D> Clone for WorthQueryInstalledDomainHandle<D> {
    fn clone(&self) -> Self {
        Self {
            authority: Arc::clone(&self.authority),
            marker: PhantomData,
        }
    }
}

impl<D> WorthQueryInstalledDomainHandle<D> {
    pub(crate) fn mint(authority: Arc<WorthQueryInstalledDomainAuthority>) -> Self {
        Self {
            authority,
            marker: PhantomData,
        }
    }

    pub fn package_identity(&self) -> &WorthQueryDomainPackageIdentity {
        self.authority.package_identity()
    }
    pub fn domain_key(&self) -> &'static str {
        self.authority.domain_key()
    }
    pub fn display_name(&self) -> &'static str {
        self.authority.display_name()
    }
    pub fn installation_generation(&self) -> WorthQueryDomainInstallationGeneration {
        self.authority.installation_generation()
    }
    pub fn installation_identity(&self) -> &str {
        self.authority.installation_identity()
    }
    pub fn runtime_provenance(&self) -> crate::runtime::WorthQueryRuntimeProvenance {
        crate::runtime::WorthQueryRuntimeProvenance::from_authority(
            self.authority.runtime_authority(),
        )
    }
    /// Observes whether this exact installed authority is still the current
    /// generation. Consumers may branch on freshness but cannot refresh or
    /// reconstruct authority from this observation.
    pub fn installation_is_current(&self) -> bool {
        self.authority.is_current_installation_generation()
    }
    pub fn authority(&self) -> &WorthQueryInstalledDomainAuthority {
        &self.authority
    }
    pub(crate) fn authority_arc(&self) -> Arc<WorthQueryInstalledDomainAuthority> {
        Arc::clone(&self.authority)
    }

    pub fn contributions(
        &self,
        runtime: &crate::runtime::WorthQueryRuntime,
    ) -> Result<
        crate::domain_capabilities::WorthQueryInstalledDomainContributionSurface,
        WorthQueryDomainHandleDenial,
    >
    where
        D: 'static,
    {
        runtime.validate_installed_domain_handle(self)?;
        Ok(
            crate::domain_capabilities::WorthQueryInstalledDomainContributionSurface::new(
                self.authority_arc(),
            ),
        )
    }

    pub fn contributions_in(
        &self,
        workspace: &crate::runtime::WorthQueryWorkspace,
    ) -> Result<
        crate::domain_capabilities::WorthQueryInstalledDomainContributionSurface,
        WorthQueryDomainHandleDenial,
    >
    where
        D: 'static,
    {
        workspace.validate_installed_domain_witness::<D>(&self.authority_witness())?;
        Ok(
            crate::domain_capabilities::WorthQueryInstalledDomainContributionSurface::new(
                self.authority_arc(),
            ),
        )
    }

    pub fn authority_witness(&self) -> WorthQueryInstalledDomainAuthorityWitness {
        WorthQueryInstalledDomainAuthorityWitness::from_authority(self.authority_arc())
    }
    pub fn graph_read_operation(
        &self,
        definition: &WorthQueryDomainGraphReadOperationDefinition,
    ) -> WorthQueryInstalledGraphReadOperation {
        let authority = self.authority_witness();
        WorthQueryInstalledGraphReadOperation::new(
            definition.declare_for_installed_authority(&authority),
            authority,
        )
    }
    pub fn rebind_request(&self) -> WorthQueryDomainRebindRequest<D> {
        WorthQueryDomainRebindRequest::new(self.authority_witness())
    }

    pub fn declarations<C>(
        &self,
        runtime: &crate::runtime::WorthQueryRuntime,
        operating_context: C,
    ) -> Result<
        WorthQueryInstalledDomainDeclarationContext<D, C>,
        WorthQueryInstalledDomainDeclarationContextDenial,
    >
    where
        D: crate::application::WorthQueryDomainEntryMarker + 'static,
        C: crate::application::WorthQueryDomainOperatingContext<D>,
    {
        runtime
            .validate_installed_domain_handle(self)
            .map_err(WorthQueryInstalledDomainDeclarationContextDenial::handle)?;
        WorthQueryInstalledDomainDeclarationContext::admit(
            self.authority_witness(),
            operating_context,
        )
    }

    pub fn declarations_in<C>(
        &self,
        workspace: &crate::runtime::WorthQueryWorkspace,
        operating_context: C,
    ) -> Result<
        WorthQueryInstalledDomainDeclarationContext<D, C>,
        WorthQueryInstalledDomainDeclarationContextDenial,
    >
    where
        D: crate::application::WorthQueryDomainEntryMarker + 'static,
        C: crate::application::WorthQueryDomainOperatingContext<D>,
    {
        let witness = self.authority_witness();
        workspace
            .validate_installed_domain_witness::<D>(&witness)
            .map_err(WorthQueryInstalledDomainDeclarationContextDenial::handle)?;
        WorthQueryInstalledDomainDeclarationContext::admit(witness, operating_context)
    }
}
