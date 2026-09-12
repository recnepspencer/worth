use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::{ApplicationMutationOutputPosture, ApplicationMutationScopeResolutionMode};

/// Portable descriptive meaning. It carries no native binding or execution authority.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationMutationDescription {
    parts: ApplicationMutationDescriptionParts,
}

/// Untrusted transport fields, validated when their schema declaration is admitted.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationMutationDescriptionParts {
    pub binding_identity: WorthQueryPortableTypeIdentity,
    pub operation: String,
    pub input_identity: WorthQueryPortableTypeIdentity,
    pub result_identity: WorthQueryPortableTypeIdentity,
    pub denial_identity: WorthQueryPortableTypeIdentity,
    pub scope: ApplicationMutationScopeDescription,
    pub output_roles: Vec<ApplicationMutationOutputRoleDescription>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationMutationScopeDescription {
    pub entity: String,
    pub aspect: String,
    pub field: String,
    pub resolution: ApplicationMutationScopeResolutionMode,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationMutationOutputRoleDescription {
    pub name: String,
    pub entity: String,
    pub posture: ApplicationMutationOutputPosture,
}

impl ApplicationMutationDescription {
    pub fn from_untrusted_parts(parts: ApplicationMutationDescriptionParts) -> Self {
        Self { parts }
    }

    pub fn parts(&self) -> &ApplicationMutationDescriptionParts {
        &self.parts
    }

    pub fn into_parts(self) -> ApplicationMutationDescriptionParts {
        self.parts
    }

    pub fn binding_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.parts.binding_identity
    }

    pub fn operation(&self) -> &str {
        &self.parts.operation
    }

    pub fn input_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.parts.input_identity
    }

    pub fn result_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.parts.result_identity
    }

    pub fn denial_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.parts.denial_identity
    }

    pub fn scope(&self) -> &ApplicationMutationScopeDescription {
        &self.parts.scope
    }

    pub fn output_roles(&self) -> &[ApplicationMutationOutputRoleDescription] {
        &self.parts.output_roles
    }
}
