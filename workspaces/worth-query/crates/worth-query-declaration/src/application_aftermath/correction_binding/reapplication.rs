//! Typed reapplication of the original source through its ordinary admission owner.
use super::ApplicationCorrectionBinding;
use crate::application_capability::{
    ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRequest,
};
use crate::application_operation::{
    ApplicationCapabilityMutationBinding, ApplicationConventionalAdmission,
    ApplicationMutationAdmissionBinding, ApplicationMutationBinding, ApplicationMutationInputScope,
    ApplicationMutationScopeBinding, ApplicationMutationScopeResolution,
};
use crate::application_schema::ApplicationSchema;
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use std::marker::PhantomData;

/// Reifies a correction's exact original source binding and its ordinary admission mode.
///
/// A correction opts in through its typed dispatch. The visitor receives the
/// bounds needed for fresh source admission, while retained input selection
/// remains inside the runtime and exposes no replacement-input callback.
pub trait ApplicationReapplicationVisitor<S: ApplicationSchema>: Sized {
    /// Result of resolving the declared source reapplication support.
    type Output;
    /// Handles a correction that has not declared source reapplication support.
    fn unavailable(self) -> Self::Output;
    /// Observes declared support without assuming either ordinary admission mode.
    fn declared<C: ApplicationCorrectionBinding<S>>(self) -> Self::Output {
        self.unavailable()
    }
    /// Resolves the original source through its conventional mutation admission.
    fn conventional<C: ApplicationCorrectionBinding<S>>(self) -> Self::Output
    where
        C::Source:
            ApplicationMutationAdmissionBinding<S, Admission = ApplicationConventionalAdmission>,
        <C::Source as ApplicationMutationBinding<S>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        self.declared::<C>()
    }
    /// Resolves the original source through its declared capability admission.
    fn capability<C, Capability>(self) -> Self::Output
    where
        C: ApplicationCorrectionBinding<S>,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = S> + 'static,
        C::Source: ApplicationCapabilityMutationBinding<S, Capability = Capability>,
        <C::Source as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source> + ApplicationCapabilityRequest<S, Capability, Scope = <<C::Source as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope>,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<S, <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity>,
    {
        self.declared::<C>()
    }
}

/// Typed opt-in dispatch restricted to a correction's own original source binding.
///
/// Its private construction prevents substituting another binding. Selecting a
/// mode declares support; the runtime must still disclose retained material and
/// perform fresh ordinary admission before executing the exact original input.
pub struct ApplicationReapplicationDispatch<S, C, V> {
    visitor: V,
    marker: PhantomData<fn() -> (S, C)>,
}

/// Visits one correction's declared support for reapplying its original source input.
pub fn dispatch_application_reapplication<S, C, V>(visitor: V) -> V::Output
where
    S: ApplicationSchema,
    C: ApplicationCorrectionBinding<S>,
    V: ApplicationReapplicationVisitor<S>,
{
    C::visit_reapplication(ApplicationReapplicationDispatch {
        visitor,
        marker: PhantomData,
    })
}

impl<S, C, V> ApplicationReapplicationDispatch<S, C, V>
where
    S: ApplicationSchema,
    C: ApplicationCorrectionBinding<S>,
    V: ApplicationReapplicationVisitor<S>,
{
    /// Declines source reapplication for this correction association.
    pub fn unavailable(self) -> V::Output {
        self.visitor.unavailable()
    }
    /// Enables reapplication through the original source's conventional admission.
    pub fn conventional(self) -> V::Output
    where
        C::Source:
            ApplicationMutationAdmissionBinding<S, Admission = ApplicationConventionalAdmission>,
        <C::Source as ApplicationMutationBinding<S>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        self.visitor.conventional::<C>()
    }
    /// Enables reapplication through the original source's exact declared capability.
    pub fn capability<Capability>(self) -> V::Output
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = S> + 'static,
        C::Source: ApplicationCapabilityMutationBinding<S, Capability = Capability>,
        <C::Source as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source> + ApplicationCapabilityRequest<S, Capability, Scope = <<C::Source as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope>,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<S, <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity>,
    {
        self.visitor.capability::<C, Capability>()
    }
}

pub(super) fn declared_source<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>>(
) -> Option<WorthQueryPortableTypeIdentity> {
    struct Source;
    impl<S: ApplicationSchema> ApplicationReapplicationVisitor<S> for Source {
        type Output = Option<WorthQueryPortableTypeIdentity>;
        fn unavailable(self) -> Self::Output {
            None
        }
        fn declared<C: ApplicationCorrectionBinding<S>>(self) -> Self::Output {
            Some(
                C::Source::descriptor()
                    .description()
                    .binding_identity()
                    .clone(),
            )
        }
    }
    dispatch_application_reapplication::<S, C, _>(Source)
}
