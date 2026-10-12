use bank_server::{
    BankAuthenticatedPrincipal, BankAuthenticationBoundary, BankIdentityRuntime, BankPrincipalSeed,
    BankWorldSeed,
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;

use crate::adapter::AuthentikOidcAdapter;
use crate::authorization::{
    AuthentikAuthorizationCallback, AuthentikAuthorizationRequest, AuthentikPendingAuthorization,
};
use crate::configuration::AuthentikOidcConfiguration;
use crate::credential::AuthentikOidcCredential;
use crate::error::{AuthentikBankAuthenticationError, AuthentikBankIdentityBuildError};

pub struct AuthentikBankIdentity {
    runtime: BankIdentityRuntime,
    authentication: BankAuthenticationBoundary<AuthentikOidcAdapter>,
}

impl AuthentikBankIdentity {
    pub async fn install(
        configuration: AuthentikOidcConfiguration,
        seeds: impl IntoIterator<Item = BankPrincipalSeed>,
        scope: &WorthQueryRequestScope,
    ) -> Result<Self, AuthentikBankIdentityBuildError> {
        let bank_configuration = configuration
            .bank_authentication_configuration()
            .map_err(AuthentikBankIdentityBuildError::Configuration)?;
        let adapter = AuthentikOidcAdapter::discover(configuration, scope)
            .await
            .map_err(AuthentikBankIdentityBuildError::Adapter)?;
        Self::install_with_adapter(bank_configuration, adapter, seeds)
    }

    pub async fn install_world(
        configuration: AuthentikOidcConfiguration,
        seed: BankWorldSeed,
        scope: &WorthQueryRequestScope,
    ) -> Result<Self, AuthentikBankIdentityBuildError> {
        let bank_configuration = configuration
            .bank_authentication_configuration()
            .map_err(AuthentikBankIdentityBuildError::Configuration)?;
        let adapter = AuthentikOidcAdapter::discover(configuration, scope)
            .await
            .map_err(AuthentikBankIdentityBuildError::Adapter)?;
        let runtime = BankIdentityRuntime::install_world(seed)
            .map_err(AuthentikBankIdentityBuildError::Runtime)?;
        let authentication = runtime
            .admit_authentication_adapter(bank_configuration, adapter)
            .map_err(AuthentikBankIdentityBuildError::AuthenticationBoundary)?;
        Ok(Self {
            runtime,
            authentication,
        })
    }

    pub(crate) async fn install_for_cold_certification_for_test(
        configuration: AuthentikOidcConfiguration,
        seeds: impl IntoIterator<Item = BankPrincipalSeed>,
        scope: &WorthQueryRequestScope,
    ) -> Result<Self, AuthentikBankIdentityBuildError> {
        let bank_configuration = configuration
            .bank_authentication_configuration()
            .map_err(AuthentikBankIdentityBuildError::Configuration)?;
        let adapter =
            AuthentikOidcAdapter::discover_for_cold_certification_for_test(configuration, scope)
                .await
                .map_err(AuthentikBankIdentityBuildError::Adapter)?;
        Self::install_with_adapter(bank_configuration, adapter, seeds)
    }

    pub(crate) async fn install_world_for_cold_certification_for_test(
        configuration: AuthentikOidcConfiguration,
        seed: BankWorldSeed,
        scope: &WorthQueryRequestScope,
    ) -> Result<Self, AuthentikBankIdentityBuildError> {
        let bank_configuration = configuration
            .bank_authentication_configuration()
            .map_err(AuthentikBankIdentityBuildError::Configuration)?;
        let adapter =
            AuthentikOidcAdapter::discover_for_cold_certification_for_test(configuration, scope)
                .await
                .map_err(AuthentikBankIdentityBuildError::Adapter)?;
        let runtime = BankIdentityRuntime::install_world(seed)
            .map_err(AuthentikBankIdentityBuildError::Runtime)?;
        let authentication = runtime
            .admit_authentication_adapter(bank_configuration, adapter)
            .map_err(AuthentikBankIdentityBuildError::AuthenticationBoundary)?;
        Ok(Self {
            runtime,
            authentication,
        })
    }

    fn install_with_adapter(
        bank_configuration: bank_server::BankAuthenticationConfiguration,
        adapter: AuthentikOidcAdapter,
        seeds: impl IntoIterator<Item = BankPrincipalSeed>,
    ) -> Result<Self, AuthentikBankIdentityBuildError> {
        let runtime = BankIdentityRuntime::install(seeds)
            .map_err(AuthentikBankIdentityBuildError::Runtime)?;
        let authentication = runtime
            .admit_authentication_adapter(bank_configuration, adapter)
            .map_err(AuthentikBankIdentityBuildError::AuthenticationBoundary)?;
        Ok(Self {
            runtime,
            authentication,
        })
    }

    pub async fn begin_authorization(&self) -> AuthentikAuthorizationRequest {
        self.authentication.adapter().begin_authorization().await
    }

    pub async fn authenticate_callback(
        &self,
        pending: AuthentikPendingAuthorization,
        callback: AuthentikAuthorizationCallback,
        scope: &WorthQueryRequestScope,
    ) -> Result<BankAuthenticatedPrincipal, AuthentikBankAuthenticationError> {
        let credential = self
            .finish_authorization(pending, callback, scope)
            .await
            .map_err(AuthentikBankAuthenticationError::AuthorizationFlow)?;
        self.authenticate_credential(credential, scope).await
    }

    pub async fn finish_authorization(
        &self,
        pending: AuthentikPendingAuthorization,
        callback: AuthentikAuthorizationCallback,
        scope: &WorthQueryRequestScope,
    ) -> Result<AuthentikOidcCredential, crate::error::AuthentikOidcFlowError> {
        self.authentication
            .adapter()
            .finish_authorization(pending, callback, scope)
            .await
    }

    pub async fn authenticate_credential(
        &self,
        credential: AuthentikOidcCredential,
        scope: &WorthQueryRequestScope,
    ) -> Result<BankAuthenticatedPrincipal, AuthentikBankAuthenticationError> {
        self.runtime
            .authenticate_with(&self.authentication, credential, scope)
            .await
            .map_err(AuthentikBankAuthenticationError::PrincipalAdmission)
    }

    pub async fn revoke_credential(
        &self,
        credential: &AuthentikOidcCredential,
        scope: &WorthQueryRequestScope,
    ) -> Result<(), crate::error::AuthentikOidcFlowError> {
        self.authentication
            .adapter()
            .revoke_credential(credential, scope)
            .await
    }

    pub fn mapped_principal_count(&self) -> usize {
        self.runtime.mapped_principal_count()
    }

    pub const fn runtime(&self) -> &BankIdentityRuntime {
        &self.runtime
    }

    pub fn jwks_refresh_count(&self) -> usize {
        self.authentication.adapter().jwks_refresh_count()
    }
}
