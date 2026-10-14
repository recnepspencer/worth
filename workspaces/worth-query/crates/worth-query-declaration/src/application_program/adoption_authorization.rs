//! Program-owned capability meaning for inspecting and adopting revisions.

use super::ApplicationProgramRevision;
use crate::application_capability::{
    ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRequest,
};
use crate::application_schema::{
    ApplicationIdentityScalarValueBinding, ApplicationOperationMarkerIdentity,
    ApplicationPrincipalBindingRef, ApplicationSchema, ApplicationStructuredValueBinding,
};

/// Pure input to the program's capability projection. It grants no authority.
pub struct ApplicationProgramAdoptionContext<'a> {
    pub source: &'a ApplicationProgramRevision,
    pub target: &'a ApplicationProgramRevision,
}

/// The existing capability contract and principal mapping governing adoption.
/// Input construction is pure; installed capability admission decides permission.
pub trait ApplicationProgramAdoptionBinding<Schema: ApplicationSchema>: 'static {
    const IDENTITY: &'static str;
    type Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema>;
    type Input: ApplicationCapabilityRequest<Schema, Self::Capability> + 'static;
    type Operation: ApplicationOperationMarkerIdentity<Schema, InputBinding = Self::InputBinding>
        + 'static;
    type InputBinding: ApplicationStructuredValueBinding<Value = Self::Input>;
    type PrincipalBinding: 'static;
    type Mapping: 'static;
    type Principal: 'static;
    type PrincipalIdentity: Clone + 'static;
    type PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<
        Value = Self::PrincipalIdentity,
    >;

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Self::PrincipalBinding,
        Self::Mapping,
        Self::Principal,
        Self::PrincipalIdentity,
        Self::PrincipalIdentityBinding,
    >;
    fn input(context: ApplicationProgramAdoptionContext<'_>) -> Self::Input;
}

/// Resolves one typed declaration without teaching Declaration about runtimes.
pub trait ApplicationProgramAdoptionAuthorizationVisitor<Schema: ApplicationSchema>: Sized {
    type Output;
    fn unavailable(self) -> Self::Output;
    fn capability<Binding: ApplicationProgramAdoptionBinding<Schema>>(self) -> Self::Output;
}

/// Canonical descriptive identity; never an admission or publication permit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationProgramAdoptionAuthorizationDeclaration {
    binding: &'static str,
    canonical: String,
}

impl ApplicationProgramAdoptionAuthorizationDeclaration {
    pub fn from_binding<
        Schema: ApplicationSchema,
        Binding: ApplicationProgramAdoptionBinding<Schema>,
    >() -> Self {
        use crate::application_capability::ApplicationCapabilityRef;
        use crate::application_schema::ApplicationOperationRef;
        let capability =
            ApplicationCapabilityRef::<Schema, Binding::Capability>::from_declaration();
        let operation =
            ApplicationOperationRef::<Schema, Binding::Operation, Binding::Input>::from_declaration(
            );
        let principal = Binding::principal_binding();
        Self {
            binding: Binding::IDENTITY,
            canonical: super::semantic_encoding::framed_record(
                "program-adoption-authorization",
                [
                    ("binding", Binding::IDENTITY.to_owned()),
                    ("capability", capability.name().to_owned()),
                    (
                        "capability-type",
                        capability.marker_identity().as_str().to_owned(),
                    ),
                    ("operation", operation.name().to_owned()),
                    ("input", operation.input_identity().as_str().to_owned()),
                    ("principal-binding", principal.name().to_owned()),
                ],
            ),
        }
    }

    pub(super) const fn binding_identity(&self) -> &'static str {
        self.binding
    }

    pub fn canonical_meaning(&self) -> &str {
        &self.canonical
    }
}

pub(super) struct DeclarationVisitor;
impl<Schema: ApplicationSchema> ApplicationProgramAdoptionAuthorizationVisitor<Schema>
    for DeclarationVisitor
{
    type Output = Option<ApplicationProgramAdoptionAuthorizationDeclaration>;
    fn unavailable(self) -> Self::Output {
        None
    }
    fn capability<Binding: ApplicationProgramAdoptionBinding<Schema>>(self) -> Self::Output {
        Some(ApplicationProgramAdoptionAuthorizationDeclaration::from_binding::<Schema, Binding>())
    }
}
