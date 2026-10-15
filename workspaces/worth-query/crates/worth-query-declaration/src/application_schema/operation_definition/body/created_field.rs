//! Portable initializer values and their exact read/write correspondence.
use crate::{
    application_schema::{
        ApplicationFieldBindingLocus, ApplicationOperationDecisionReadTarget,
        ApplicationOperationProgramTarget,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};
use worth_foundational::facade::{AspectValue, ScalarAspectType};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationBodyCreatedField {
    Literal {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        value: AspectValue,
    },
    Input {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
        member: String,
    },
    OptionalInput {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
        member: String,
    },
    ScopeField {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
        source: ApplicationFieldBindingLocus,
    },
    PrincipalIdentity {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
        mutation_binding: String,
        principal_binding: String,
        source: ApplicationFieldBindingLocus,
    },
}
impl ApplicationOperationBodyCreatedField {
    pub fn field(&self) -> &ApplicationFieldBindingLocus {
        match self {
            Self::Literal { field, .. }
            | Self::Input { field, .. }
            | Self::OptionalInput { field, .. }
            | Self::ScopeField { field, .. }
            | Self::PrincipalIdentity { field, .. } => field,
        }
    }
    pub fn binding(&self) -> &WorthQueryPortableTypeIdentity {
        match self {
            Self::Literal { binding, .. }
            | Self::Input { binding, .. }
            | Self::OptionalInput { binding, .. }
            | Self::ScopeField { binding, .. }
            | Self::PrincipalIdentity { binding, .. } => binding,
        }
    }
    pub fn literal_value(&self) -> Option<&AspectValue> {
        match self {
            Self::Literal { value, .. } => Some(value),
            _ => None,
        }
    }
    pub fn scalar_family(&self) -> ScalarAspectType {
        match self {
            Self::Literal { value, .. } => value.value_family(),
            Self::Input { scalar_family, .. }
            | Self::OptionalInput { scalar_family, .. }
            | Self::ScopeField { scalar_family, .. }
            | Self::PrincipalIdentity { scalar_family, .. } => *scalar_family,
        }
    }
    pub fn input_member(&self) -> Option<&str> {
        match self {
            Self::Input { member, .. } | Self::OptionalInput { member, .. } => Some(member),
            _ => None,
        }
    }
    pub fn source_field(&self) -> Option<&ApplicationFieldBindingLocus> {
        match self {
            Self::ScopeField { source, .. } => Some(source),
            _ => None,
        }
    }
    pub fn principal_source(&self) -> Option<&ApplicationFieldBindingLocus> {
        match self {
            Self::PrincipalIdentity { source, .. } => Some(source),
            _ => None,
        }
    }
    pub fn input_is_optional(&self) -> bool {
        matches!(self, Self::OptionalInput { .. })
    }
    pub fn source_read(&self) -> Option<ApplicationOperationDecisionReadTarget> {
        self.source_field()
            .or_else(|| self.principal_source())
            .map(|source| ApplicationOperationDecisionReadTarget::Field {
                entity: source.entity().to_owned(),
                aspect: source.aspect().to_owned(),
                field: source.field().to_owned(),
            })
    }
    pub fn write_target(&self) -> ApplicationOperationProgramTarget {
        let field = self.field();
        ApplicationOperationProgramTarget::Write {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
        }
    }
}
