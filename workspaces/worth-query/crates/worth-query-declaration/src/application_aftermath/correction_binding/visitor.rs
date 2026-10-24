use super::{ApplicationCorrectionBinding, ApplicationInverseBinding};
use crate::application_capability::{
    ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRequest,
};
use crate::application_operation::{
    ApplicationCapabilityMutationBinding, ApplicationConventionalAdmission,
    ApplicationMutationAdmissionBinding, ApplicationMutationBinding, ApplicationMutationInputScope,
    ApplicationMutationScopeResolution,
};
use crate::application_schema::ApplicationSchema;

/// Reifies the declared inverse's ordinary admission mode without runtime callbacks.
pub trait ApplicationCorrectionVisitor<Schema: ApplicationSchema>: Sized {
    type Output;
    fn unavailable(self) -> Self::Output;

    /// Observe declared meaning without admitting or executing its inverse.
    fn declared<Correction: ApplicationCorrectionBinding<Schema>>(self) -> Self::Output {
        self.unavailable()
    }

    fn conventional<Correction: ApplicationCorrectionBinding<Schema>>(self) -> Self::Output
    where
        ApplicationInverseBinding<Schema, Correction>: ApplicationMutationAdmissionBinding<
            Schema, Admission = ApplicationConventionalAdmission,
        >,
        <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<Schema, ApplicationInverseBinding<Schema, Correction>>,
        <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::ScopeBinding: ApplicationMutationScopeResolution<
            Schema, <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
    >{
        self.declared::<Correction>()
    }

    fn capability<Correction, Capability>(self) -> Self::Output
    where
        Correction: ApplicationCorrectionBinding<Schema>,
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        ApplicationInverseBinding<Schema, Correction>: ApplicationCapabilityMutationBinding<Schema, Capability = Capability>,
        <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<Schema, ApplicationInverseBinding<Schema, Correction>> + ApplicationCapabilityRequest<
            Schema, Capability,
            Scope = <<ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::ScopeBinding as crate::application_operation::ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::ScopeBinding: ApplicationMutationScopeResolution<
            Schema, <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
    >{
        self.declared::<Correction>()
    }
}
