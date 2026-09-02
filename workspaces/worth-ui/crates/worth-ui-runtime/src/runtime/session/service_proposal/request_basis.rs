use core::num::NonZeroU64;

#[path = "request_basis/admitted_intent.rs"]
mod admitted_intent;
#[cfg(any(test, feature = "certification-support"))]
#[path = "request_basis/portal_certification.rs"]
mod portal_certification;
#[path = "request_basis/portal_dismissal.rs"]
mod portal_dismissal;
#[path = "request_basis/portal_exit_terminal.rs"]
mod portal_exit_terminal;

pub(in crate::runtime) use admitted_intent::UiAdmittedIntentServiceRequestAuthority;
#[cfg(any(test, feature = "certification-support"))]
pub(in crate::runtime) use portal_certification::UiPortalCertificationServiceRequestAuthority;
pub(in crate::runtime) use portal_dismissal::UiPortalDismissalServiceRequestAuthority;
pub(in crate::runtime) use portal_exit_terminal::UiPortalExitTerminalServiceRequestAuthority;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::runtime) struct UiServiceRequestIdentity(NonZeroU64);

impl UiServiceRequestIdentity {
    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::runtime) fn for_test(value: u64) -> Self {
        Self(NonZeroU64::new(value).expect("test request identity must be non-zero"))
    }

    pub(in crate::runtime) const fn diagnostic_value(self) -> u64 {
        self.0.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::runtime) struct UiServiceSourceOrder(NonZeroU64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::runtime) struct UiServiceCancellationIdentity(NonZeroU64);

impl UiServiceCancellationIdentity {
    #[cfg(test)]
    pub(in crate::runtime) fn for_test(value: u64) -> Self {
        Self(NonZeroU64::new(value).expect("test cancellation identity must be non-zero"))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(in crate::runtime) struct UiServiceResourceBudgetIdentity(NonZeroU64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum UiServiceRequestOrigin {
    AdmittedIntent,
    HostObservation,
    #[cfg(test)]
    Rebind,
    ServiceContinuation,
    RuntimePolicy,
    #[cfg(test)]
    Teardown,
}

pub(in crate::runtime) mod sealed {
    pub trait Sealed {}
}

/// Implemented in this owner module for each concrete inherited producer
/// authority. Downstream runtime modules cannot add authority implementations.
pub(in crate::runtime) trait UiServiceRequestOriginAuthority:
    sealed::Sealed
{
    fn service_request_origin(&self) -> UiServiceRequestOrigin;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::runtime) struct UiServiceSurfaceBasis {
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    host_surface: worth_ui_host_contract::UiHostSurfaceIdentity,
    binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
}

worth_proof::binding_axes! {
    pub(in crate::runtime) struct UiServiceRequestCoherenceAxes {
        pub(in crate::runtime) application:
            crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity => Application,
        pub(in crate::runtime) semantic_surface:
            worth_ui_host_contract::UiSemanticSurfaceIdentity => SemanticSurface,
        pub(in crate::runtime) host_surface:
            worth_ui_host_contract::UiHostSurfaceIdentity => HostSurface,
        pub(in crate::runtime) binding:
            worth_ui_host_contract::UiSurfaceBindingGeneration => Binding,
        pub(in crate::runtime) presentation:
            Option<worth_ui_host_contract::UiHostObservationPresentationBasis> => Presentation,
        pub(in crate::runtime) origin: UiServiceRequestOrigin => Origin,
        pub(in crate::runtime) causal_parent:
            Option<UiServiceRequestIdentity> => CausalParent,
        pub(in crate::runtime) causal_root: UiServiceRequestIdentity => CausalRoot,
        pub(in crate::runtime) source_order: UiServiceSourceOrder => SourceOrder,
        pub(in crate::runtime) cancellation: UiServiceCancellationIdentity => Cancellation,
        pub(in crate::runtime) resource_budget: UiServiceResourceBudgetIdentity => ResourceBudget,
    }
    drift pub(crate) enum UiServiceRequestCoherenceDrift;
}

pub(in crate::runtime) type UiServiceRequestCoherence =
    worth_proof::Binding<UiServiceRequestCoherenceAxes>;

pub(in crate::runtime) struct UiServiceRequestBasis<Authority>
where
    Authority: UiServiceRequestOriginAuthority,
{
    identity: UiServiceRequestIdentity,
    causal_parent: Option<UiServiceRequestIdentity>,
    causal_root: UiServiceRequestIdentity,
    application: crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
    surface: UiServiceSurfaceBasis,
    presentation: Option<worth_ui_host_contract::UiHostObservationPresentationBasis>,
    source_order: UiServiceSourceOrder,
    cancellation: UiServiceCancellationIdentity,
    resource_budget: UiServiceResourceBudgetIdentity,
    authority: Authority,
}

struct UiServiceRequestBasisInput<Authority>
where
    Authority: UiServiceRequestOriginAuthority,
{
    identity: UiServiceRequestIdentity,
    causal_parent: Option<UiServiceRequestIdentity>,
    causal_root: UiServiceRequestIdentity,
    application: crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
    surface: UiServiceSurfaceBasis,
    presentation: Option<worth_ui_host_contract::UiHostObservationPresentationBasis>,
    source_order: UiServiceSourceOrder,
    cancellation: UiServiceCancellationIdentity,
    resource_budget: UiServiceResourceBudgetIdentity,
    authority: Authority,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiServiceRequestBasisDenial {
    IdentityExhausted,
    RootIdentityMismatch,
    ChildRootIdentityMismatch,
    SelfParent,
    PresentationBindingChanged,
    PresentationSurfaceChanged,
}

impl UiServiceSurfaceBasis {
    pub(in crate::runtime) fn from_coherence(coherence: &UiServiceRequestCoherence) -> Self {
        Self {
            semantic_surface: coherence.axes().semantic_surface,
            host_surface: coherence.axes().host_surface,
            binding: coherence.axes().binding,
        }
    }

    pub(in crate::runtime) const fn semantic_surface(
        self,
    ) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }

    pub(in crate::runtime) const fn host_surface(
        self,
    ) -> worth_ui_host_contract::UiHostSurfaceIdentity {
        self.host_surface
    }

    pub(in crate::runtime) const fn binding(
        self,
    ) -> worth_ui_host_contract::UiSurfaceBindingGeneration {
        self.binding
    }
}

impl<Authority> UiServiceRequestBasis<Authority>
where
    Authority: UiServiceRequestOriginAuthority,
{
    fn seal(
        input: UiServiceRequestBasisInput<Authority>,
    ) -> Result<Self, UiServiceRequestBasisDenial> {
        validate_causal_basis(input.identity, input.causal_parent, input.causal_root)?;
        validate_presentation_binding(input.surface, input.presentation)?;
        Ok(Self {
            identity: input.identity,
            causal_parent: input.causal_parent,
            causal_root: input.causal_root,
            application: input.application,
            surface: input.surface,
            presentation: input.presentation,
            source_order: input.source_order,
            cancellation: input.cancellation,
            resource_budget: input.resource_budget,
            authority: input.authority,
        })
    }

    pub(in crate::runtime) const fn identity(&self) -> UiServiceRequestIdentity {
        self.identity
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn causal_parent(&self) -> Option<UiServiceRequestIdentity> {
        self.causal_parent
    }

    #[cfg(test)]
    pub(in crate::runtime) fn application(
        &self,
    ) -> &crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity {
        &self.application
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn surface(&self) -> UiServiceSurfaceBasis {
        self.surface
    }

    #[cfg(test)]
    pub(in crate::runtime) const fn presentation(
        &self,
    ) -> Option<worth_ui_host_contract::UiHostObservationPresentationBasis> {
        self.presentation
    }

    #[cfg(test)]
    pub(in crate::runtime) fn origin(&self) -> UiServiceRequestOrigin {
        self.authority.service_request_origin()
    }

    pub(in crate::runtime) fn coherence(&self) -> UiServiceRequestCoherence {
        worth_proof::Binding::new(UiServiceRequestCoherenceAxes {
            application: self.application.clone(),
            semantic_surface: self.surface.semantic_surface,
            host_surface: self.surface.host_surface,
            binding: self.surface.binding,
            presentation: self.presentation,
            origin: self.authority.service_request_origin(),
            causal_parent: self.causal_parent,
            causal_root: self.causal_root,
            source_order: self.source_order,
            cancellation: self.cancellation,
            resource_budget: self.resource_budget,
        })
    }
}

fn validate_causal_basis(
    identity: UiServiceRequestIdentity,
    parent: Option<UiServiceRequestIdentity>,
    root: UiServiceRequestIdentity,
) -> Result<(), UiServiceRequestBasisDenial> {
    match parent {
        None if root != identity => Err(UiServiceRequestBasisDenial::RootIdentityMismatch),
        Some(_) if root == identity => Err(UiServiceRequestBasisDenial::ChildRootIdentityMismatch),
        Some(parent) if parent == identity => Err(UiServiceRequestBasisDenial::SelfParent),
        _ => Ok(()),
    }
}

fn validate_presentation_binding(
    surface: UiServiceSurfaceBasis,
    presentation: Option<worth_ui_host_contract::UiHostObservationPresentationBasis>,
) -> Result<(), UiServiceRequestBasisDenial> {
    if let Some(presentation) = presentation {
        if presentation.host_surface() != surface.host_surface() {
            return Err(UiServiceRequestBasisDenial::PresentationSurfaceChanged);
        }
        if presentation.binding() != surface.binding() {
            return Err(UiServiceRequestBasisDenial::PresentationBindingChanged);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(feature = "certification-support")]
mod fixture_certification_support;

#[cfg(feature = "certification-support")]
pub(super) use fixture_certification_support::{
    fixture_application_generation, fixture_service_request_coherence_in,
};

#[cfg(test)]
pub(super) use tests::{
    fixture_application_generation_in_session, fixture_service_request_coherence,
};
