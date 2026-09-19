use core::num::NonZeroU64;

#[derive(Debug)]
pub(in crate::runtime) struct UiPortalCertificationServiceRequestAuthority;

impl super::sealed::Sealed for UiPortalCertificationServiceRequestAuthority {}

impl super::UiServiceRequestOriginAuthority for UiPortalCertificationServiceRequestAuthority {
    fn service_request_origin(&self) -> super::UiServiceRequestOrigin {
        super::UiServiceRequestOrigin::RuntimePolicy
    }
}

impl super::UiServiceRequestBasis<UiPortalCertificationServiceRequestAuthority> {
    pub(in crate::runtime) fn from_portal_certification(
        transition: &crate::runtime::portal::UiPreparedPortalServiceTransition,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        application: crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
    ) -> Result<Self, super::UiServiceRequestBasisDenial> {
        let request = transition.request();
        let issued = NonZeroU64::new(request.idempotency().lineage())
            .ok_or(super::UiServiceRequestBasisDenial::IdentityExhausted)?;
        let identity = super::UiServiceRequestIdentity(issued);
        Self::seal(super::UiServiceRequestBasisInput {
            identity,
            causal_parent: None,
            causal_root: identity,
            application,
            surface: super::UiServiceSurfaceBasis {
                semantic_surface: request.semantic_surface(),
                host_surface: presentation.host_surface(),
                binding: presentation.binding(),
            },
            presentation: Some(presentation),
            source_order: super::UiServiceSourceOrder(issued),
            cancellation: super::UiServiceCancellationIdentity(issued),
            resource_budget: super::UiServiceResourceBudgetIdentity(issued),
            authority: UiPortalCertificationServiceRequestAuthority,
        })
    }
}
