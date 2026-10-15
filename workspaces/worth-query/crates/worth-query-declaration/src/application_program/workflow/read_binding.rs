//! Typed executable admission shape. This pure description grants no read.
use crate::application_capability::{
    ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRequest,
};
use crate::application_schema::{
    ApplicationIdentityScalarValueBinding, ApplicationOperationMarkerIdentity,
    ApplicationPrincipalBindingRef, ApplicationSchema, ApplicationStructuredValueBinding,
};
use std::any::TypeId;

/// Binds a workflow disclosure rule to the existing capability admission pipeline.
/// The caller supplies actual typed input; no workflow lookup constructs it.
///
/// ```
/// use worth_query_declaration::facade::application_program::{
///     ApplicationWorkflowSpec, ApplicationWorkflowReadBinding, ApplicationWorkflowReadRule,
/// };
/// fn subject<S,B>() -> ApplicationWorkflowReadRule
/// where S:ApplicationWorkflowSpec,B:ApplicationWorkflowReadBinding<S::Schema> {
///     ApplicationWorkflowReadRule::subject::<S,B>()
/// }
/// ```
/// A capability marker alone has no executable read-input/principal binding.
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_program::{ApplicationWorkflowSpec,ApplicationWorkflowReadRule};
/// use worth_query_declaration::facade::application_capability::ApplicationCapabilityMarkerIdentity;
/// fn marker_only<S,C>() where S:ApplicationWorkflowSpec,C:ApplicationCapabilityMarkerIdentity<Schema=S::Schema> {
///     let _ = ApplicationWorkflowReadRule::subject::<S,C>();
/// }
/// ```
pub trait ApplicationWorkflowReadBinding<Schema: ApplicationSchema>: 'static {
    const IDENTITY: &'static str;
    type Capability: ApplicationCapabilityMarkerIdentity<Schema = Schema>;
    type Input: ApplicationCapabilityRequest<Schema, Self::Capability> + 'static;
    type Operation: ApplicationOperationMarkerIdentity<Schema, InputBinding = Self::InputBinding>
        + 'static;
    type InputBinding: ApplicationStructuredValueBinding<Value = Self::Input> + 'static;
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
}

/// Current typed binding meaning; nominal stamps are process-local, never portable bytes.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationWorkflowReadBindingDeclaration {
    binding: String,
    capability: String,
    capability_type: String,
    operation: String,
    input: String,
    principal: Vec<String>,
    stamps: [TypeId; 3],
}
impl ApplicationWorkflowReadBindingDeclaration {
    pub fn from_binding<Schema: ApplicationSchema, B: ApplicationWorkflowReadBinding<Schema>>(
    ) -> Self {
        let p = B::principal_binding();
        Self {
            binding: B::IDENTITY.to_owned(),
            capability: B::Capability::IDENTIFIER.to_owned(),
            capability_type: <B::Capability as crate::portable_identity::WorthQueryPortableType>::PORTABLE_TYPE_IDENTITY.as_str().to_owned(),
            operation: B::Operation::IDENTIFIER.to_owned(),
            input: B::InputBinding::IDENTITY.as_str().to_owned(),
            principal: vec![p.name(), p.mapping_entity(), p.identity_aspect(), p.identity_field(),
                p.status_aspect(), p.status_field(), p.target_relation(), p.principal_entity(),
                p.principal_identity_aspect(), p.principal_identity_field(),
                p.principal_identity_scalar_family().canonical_name(), p.principal_identity_value_type()]
                .into_iter().map(str::to_owned).collect(),
            stamps: [TypeId::of::<B>(), TypeId::of::<B::Operation>(), TypeId::of::<B::InputBinding>()],
        }
    }
    pub fn capability(&self) -> (&str, &str) {
        (&self.capability, &self.capability_type)
    }
    pub fn operation(&self) -> &str {
        &self.operation
    }
    pub fn input(&self) -> &str {
        &self.input
    }
    /// Exact principal mapping/identity/status/target descriptors in native member order.
    pub fn principal(&self) -> &[String] {
        &self.principal
    }
    /// Matches the complete authored principal member, not just its name.
    pub fn matches_principal_member(
        &self,
        member: &crate::application_schema::ApplicationSchemaMember,
    ) -> bool {
        use crate::application_schema::ApplicationSchemaMember;
        let ApplicationSchemaMember::PrincipalBinding {
            binding,
            mapping_entity,
            identity_aspect,
            identity_field,
            status_aspect,
            status_field,
            target_relation,
            principal_entity,
            principal_identity_aspect,
            principal_identity_field,
            principal_identity_scalar_family,
            principal_identity_value_type,
        } = member
        else {
            return false;
        };
        self.principal.iter().map(String::as_str).eq([
            binding.as_str(),
            mapping_entity.as_str(),
            identity_aspect.as_str(),
            identity_field.as_str(),
            status_aspect.as_str(),
            status_field.as_str(),
            target_relation.as_str(),
            principal_entity.as_str(),
            principal_identity_aspect.as_str(),
            principal_identity_field.as_str(),
            principal_identity_scalar_family.canonical_name(),
            principal_identity_value_type.as_str(),
        ])
    }
    /// Portable meaning excludes process-local nominal stamps.
    pub fn persistence_identity(&self) -> String {
        let fields = std::iter::once(self.binding.as_str())
            .chain([
                self.capability.as_str(),
                self.capability_type.as_str(),
                self.operation.as_str(),
                self.input.as_str(),
            ])
            .chain(self.principal.iter().map(String::as_str));
        fields.fold("2:workflow-read-binding".to_owned(), |mut text, field| {
            use std::fmt::Write;
            write!(&mut text, ":{}:{}", field.len(), field).expect("String write");
            text
        })
    }
}
