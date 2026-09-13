use super::WorthUiPresentationRequestBasis;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiPresentationAsyncDeclaration {
    request_identity: worth_query::facade::foundation::WorthQueryAsyncResourceRequestIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiPresentationAsyncDeclarationDenial {
    Identity(worth_query::facade::foundation::WorthQueryAsyncResourceRequestIdentityError),
}

impl WorthUiPresentationAsyncDeclaration {
    pub fn declare(
        basis: &WorthUiPresentationRequestBasis,
    ) -> Result<Self, WorthUiPresentationAsyncDeclarationDenial> {
        use worth_query::facade::foundation::{
            WorthQueryAsyncFailurePosture, WorthQueryAsyncLoadingPosture,
            WorthQueryAsyncResourceRequestIdentity, WorthQueryAsyncSourceFamily,
        };
        let request_identity = WorthQueryAsyncResourceRequestIdentity::declare(
            WorthQueryAsyncSourceFamily::HostResource,
            WorthQueryAsyncLoadingPosture::Blocking,
            WorthQueryAsyncFailurePosture::RetainStaleValue,
            basis.identity_parts(),
        )
        .map_err(WorthUiPresentationAsyncDeclarationDenial::Identity)?;
        Ok(Self { request_identity })
    }

    pub fn request_identity(
        &self,
    ) -> &worth_query::facade::foundation::WorthQueryAsyncResourceRequestIdentity {
        &self.request_identity
    }
}
