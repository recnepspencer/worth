//! The existing query whose current admission governs historical material.
use crate::application_query::{
    ApplicationQueryBinding, ApplicationQueryBindingDescriptor, ApplicationQueryScopeResolutionMode,
};
use crate::application_schema::ApplicationSchema;
use crate::portable_identity::WorthQueryPortableTypeIdentity as Identity;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredCorrectionDisclosure {
    parts: DeclaredCorrectionDisclosureParts,
}

/// Authority-free query association reconstructed from portable meaning.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DeclaredCorrectionDisclosureParts {
    pub binding: Identity,
    pub input: Identity,
    pub query_name: String,
    pub query: Identity,
    pub parameters: Identity,
    pub result: Identity,
    pub scope: Identity,
    pub scope_entity: String,
    pub scope_aspect: String,
    pub scope_field: String,
    pub scope_value: Identity,
    pub scope_resolution: ApplicationQueryScopeResolutionMode,
    pub scope_writable: bool,
    pub principal_binding: String,
}

impl DeclaredCorrectionDisclosure {
    pub fn of<Schema: ApplicationSchema, Binding: ApplicationQueryBinding<Schema>>() -> Self {
        Self::from_descriptor(&Binding::descriptor())
    }

    pub fn from_untrusted_parts(parts: DeclaredCorrectionDisclosureParts) -> Self {
        Self { parts }
    }
    pub fn parts(&self) -> &DeclaredCorrectionDisclosureParts {
        &self.parts
    }

    pub fn matches_descriptor(&self, descriptor: &ApplicationQueryBindingDescriptor) -> bool {
        self == &Self::from_descriptor(descriptor)
    }

    fn from_descriptor(descriptor: &ApplicationQueryBindingDescriptor) -> Self {
        let field = descriptor.scope().field();
        Self {
            parts: DeclaredCorrectionDisclosureParts {
                binding: Identity::from_untrusted(descriptor.identity().to_owned()),
                input: descriptor.input_identity().clone(),
                query_name: descriptor.query_name().to_owned(),
                query: descriptor.query_identity().clone(),
                parameters: descriptor.parameter_identity().clone(),
                result: descriptor.result_identity().clone(),
                scope: descriptor.scope_identity().clone(),
                scope_entity: field.locus().entity().to_owned(),
                scope_aspect: field.locus().aspect().to_owned(),
                scope_field: field.locus().field().to_owned(),
                scope_value: field.binding_identity().clone(),
                scope_resolution: descriptor.scope().mode(),
                scope_writable: descriptor.scope().writable(),
                principal_binding: descriptor.principal().name().to_owned(),
            },
        }
    }

    /// Stable named fields used by canonical declaration and installation identities.
    pub fn canonical_fields(&self) -> [(&'static str, &str); 14] {
        let p = &self.parts;
        [
            ("binding", p.binding.as_str()),
            ("input", p.input.as_str()),
            ("query-name", &p.query_name),
            ("query", p.query.as_str()),
            ("parameters", p.parameters.as_str()),
            ("result", p.result.as_str()),
            ("scope", p.scope.as_str()),
            ("scope-entity", &p.scope_entity),
            ("scope-aspect", &p.scope_aspect),
            ("scope-field", &p.scope_field),
            ("scope-value", p.scope_value.as_str()),
            (
                "scope-resolution",
                match p.scope_resolution {
                    ApplicationQueryScopeResolutionMode::InputField => "input-field",
                    ApplicationQueryScopeResolutionMode::PrincipalIdentity => "principal-identity",
                },
            ),
            (
                "scope-writable",
                if p.scope_writable { "true" } else { "false" },
            ),
            ("principal-binding", &p.principal_binding),
        ]
    }
}
