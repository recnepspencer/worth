//! Fix the visited binding to the contribution's declared correction.
use super::{
    ApplicationCorrectionBinding, ApplicationCorrectionVisitor, ApplicationInverseBinding,
};
use crate::application_capability::{
    ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRequest,
};
use crate::application_operation::{
    ApplicationCapabilityMutationBinding, ApplicationConventionalAdmission,
    ApplicationMutationAdmissionBinding, ApplicationMutationBinding, ApplicationMutationInputScope,
    ApplicationMutationScopeBinding, ApplicationMutationScopeResolution,
};
use crate::application_schema::ApplicationSchema;
use std::marker::PhantomData;

/// A declaration's admission choice can forward only that exact correction.
/// No callback, permission or runtime state is stored here.
///
/// Forwarding the declared binding is lawful:
/// ```
/// use worth_query_declaration::facade::{application_aftermath::*, application_schema::ApplicationSchema};
/// fn declared<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>, V: ApplicationCorrectionVisitor<S>>(
///     dispatch: ApplicationCorrectionDispatch<S, C, V>,
/// ) -> V::Output { C::visit(dispatch) }
/// ```
/// Another correction cannot replace the declaration's binding:
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::{application_aftermath::*, application_schema::ApplicationSchema};
/// fn substitute<S: ApplicationSchema, C: ApplicationCorrectionBinding<S>, Other: ApplicationCorrectionBinding<S>, V: ApplicationCorrectionVisitor<S>>(
///     dispatch: ApplicationCorrectionDispatch<S, C, V>,
/// ) -> V::Output { Other::visit(dispatch) }
/// ```
pub struct ApplicationCorrectionDispatch<Schema, Correction, Visitor> {
    visitor: Visitor,
    marker: PhantomData<fn() -> (Schema, Correction)>,
}

/// Reifies one declared correction through its chosen ordinary admission mode.
pub fn dispatch_application_correction<Schema, Correction, Visitor>(
    visitor: Visitor,
) -> Visitor::Output
where
    Schema: ApplicationSchema,
    Correction: ApplicationCorrectionBinding<Schema>,
    Visitor: ApplicationCorrectionVisitor<Schema>,
{
    Correction::visit(ApplicationCorrectionDispatch {
        visitor,
        marker: PhantomData,
    })
}

impl<Schema, Correction, Visitor> ApplicationCorrectionDispatch<Schema, Correction, Visitor>
where
    Schema: ApplicationSchema,
    Correction: ApplicationCorrectionBinding<Schema>,
    Visitor: ApplicationCorrectionVisitor<Schema>,
{
    pub fn conventional(self) -> Visitor::Output
    where
        ApplicationInverseBinding<Schema, Correction>: ApplicationMutationAdmissionBinding<
            Schema, Admission = ApplicationConventionalAdmission,
        >,
        <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<Schema, ApplicationInverseBinding<Schema, Correction>>,
        <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::ScopeBinding: ApplicationMutationScopeResolution<
            Schema, <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    {
        self.visitor.conventional::<Correction>()
    }

    pub fn capability<Capability>(self) -> Visitor::Output
    where
        Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema> + 'static,
        ApplicationInverseBinding<Schema, Correction>: ApplicationCapabilityMutationBinding<Schema, Capability = Capability>,
        <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync + ApplicationMutationInputScope<Schema, ApplicationInverseBinding<Schema, Correction>> + ApplicationCapabilityRequest<
            Schema, Capability,
            Scope = <<ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::ScopeBinding: ApplicationMutationScopeResolution<
            Schema, <ApplicationInverseBinding<Schema, Correction> as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
    {
        self.visitor.capability::<Correction, Capability>()
    }
}
