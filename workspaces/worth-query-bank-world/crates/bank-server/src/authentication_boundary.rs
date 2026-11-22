use bank_domain::schema::BankSchema;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmittedAuthenticationAdapter, WorthQueryAuthenticatedExternalPrincipal,
    WorthQueryAuthenticationAdapter, WorthQueryAuthenticationDenial,
    WorthQueryLocalProcessPrincipal, WorthQueryRequestScope,
};

pub struct BankAuthenticationBoundary<Adapter> {
    admitted: WorthQueryAdmittedAuthenticationAdapter<BankSchema, Adapter>,
}

impl<Adapter> BankAuthenticationBoundary<Adapter>
where
    Adapter: WorthQueryAuthenticationAdapter,
{
    pub(crate) fn new(
        admitted: WorthQueryAdmittedAuthenticationAdapter<BankSchema, Adapter>,
    ) -> Self {
        Self { admitted }
    }

    pub fn adapter(&self) -> &Adapter {
        self.admitted.adapter()
    }

    pub(crate) async fn authenticate(
        &self,
        credential: Adapter::Credential,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryAuthenticatedExternalPrincipal<BankSchema>, WorthQueryAuthenticationDenial>
    {
        self.admitted.authenticate(credential, scope).await
    }
}

impl BankAuthenticationBoundary<WorthQueryLocalProcessPrincipal> {
    /// Authenticates the admitted local process on the calling thread.
    pub(crate) fn authenticate_local(
        &self,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthQueryAuthenticatedExternalPrincipal<BankSchema>, WorthQueryAuthenticationDenial>
    {
        self.admitted.authenticate_local(scope)
    }
}
