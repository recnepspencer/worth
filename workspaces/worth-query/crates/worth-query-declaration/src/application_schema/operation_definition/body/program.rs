//! Ordered, portable native values for one operation's scope-bound body.

use worth_foundational::facade::{AspectValue, ScalarAspectType};

use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationOperationDecisionReadTarget,
    ApplicationOperationProgramTarget,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationBodyScopeField {
    Expect {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        value: AspectValue,
    },
    Assign {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        value: AspectValue,
    },
    AssignInput {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
        member: String,
    },
    SetOptionalInput {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
        member: String,
    },
    ClearOptional {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
    },
    RequirePresent {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
    },
}

impl ApplicationOperationBodyScopeField {
    pub fn field(&self) -> &ApplicationFieldBindingLocus {
        match self {
            Self::Expect { field, .. }
            | Self::Assign { field, .. }
            | Self::AssignInput { field, .. }
            | Self::SetOptionalInput { field, .. }
            | Self::ClearOptional { field, .. }
            | Self::RequirePresent { field, .. } => field,
        }
    }

    pub fn binding(&self) -> &WorthQueryPortableTypeIdentity {
        match self {
            Self::Expect { binding, .. }
            | Self::Assign { binding, .. }
            | Self::AssignInput { binding, .. }
            | Self::SetOptionalInput { binding, .. }
            | Self::ClearOptional { binding, .. }
            | Self::RequirePresent { binding, .. } => binding,
        }
    }

    pub fn literal_value(&self) -> Option<&AspectValue> {
        match self {
            Self::Expect { value, .. } | Self::Assign { value, .. } => Some(value),
            Self::AssignInput { .. }
            | Self::SetOptionalInput { .. }
            | Self::ClearOptional { .. }
            | Self::RequirePresent { .. } => None,
        }
    }

    pub fn scalar_family(&self) -> ScalarAspectType {
        match self {
            Self::Expect { value, .. } | Self::Assign { value, .. } => value.value_family(),
            Self::AssignInput { scalar_family, .. }
            | Self::SetOptionalInput { scalar_family, .. }
            | Self::ClearOptional { scalar_family, .. }
            | Self::RequirePresent { scalar_family, .. } => *scalar_family,
        }
    }
    pub fn input_member(&self) -> Option<&str> {
        match self {
            Self::AssignInput { member, .. } | Self::SetOptionalInput { member, .. } => {
                Some(member)
            }
            _ => None,
        }
    }

    pub fn decision_read(&self) -> ApplicationOperationDecisionReadTarget {
        let field = self.field();
        ApplicationOperationDecisionReadTarget::Field {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
        }
    }

    pub fn write_target(&self) -> Option<ApplicationOperationProgramTarget> {
        if matches!(self, Self::Expect { .. } | Self::RequirePresent { .. }) {
            return None;
        }
        let field = self.field();
        Some(ApplicationOperationProgramTarget::Write {
            entity: field.entity().to_owned(),
            aspect: field.aspect().to_owned(),
            field: field.field().to_owned(),
        })
    }
}

/// Schema-owned operation meaning, not an executable permit or caller payload.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyProgram {
    pub(super) scope_fields: Vec<ApplicationOperationBodyScopeField>,
    pub(super) creation: Option<ApplicationOperationBodyCreation>,
    pub(super) requirements: Vec<super::ApplicationOperationBodyRequirementProgram>,
}

/// One typed, candidate-local creation site in the admitted scope's context.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyCreation {
    pub(super) entity: String,
    pub(super) key: String,
    pub(super) fields: Vec<ApplicationOperationBodyCreatedField>,
}

impl ApplicationOperationBodyCreation {
    pub fn from_untrusted(
        entity: String,
        key: String,
        fields: Vec<ApplicationOperationBodyCreatedField>,
    ) -> Self {
        Self {
            entity,
            key,
            fields,
        }
    }
    pub fn entity(&self) -> &str {
        &self.entity
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn fields(&self) -> &[ApplicationOperationBodyCreatedField] {
        &self.fields
    }
}

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
    ScopeField {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
        source: ApplicationFieldBindingLocus,
    },
}

impl ApplicationOperationBodyCreatedField {
    pub fn field(&self) -> &ApplicationFieldBindingLocus {
        match self {
            Self::Literal { field, .. }
            | Self::Input { field, .. }
            | Self::ScopeField { field, .. } => field,
        }
    }
    pub fn binding(&self) -> &WorthQueryPortableTypeIdentity {
        match self {
            Self::Literal { binding, .. }
            | Self::Input { binding, .. }
            | Self::ScopeField { binding, .. } => binding,
        }
    }
    pub fn literal_value(&self) -> Option<&AspectValue> {
        match self {
            Self::Literal { value, .. } => Some(value),
            Self::Input { .. } | Self::ScopeField { .. } => None,
        }
    }
    pub fn scalar_family(&self) -> ScalarAspectType {
        match self {
            Self::Literal { value, .. } => value.value_family(),
            Self::Input { scalar_family, .. } | Self::ScopeField { scalar_family, .. } => {
                *scalar_family
            }
        }
    }
    pub fn input_member(&self) -> Option<&str> {
        match self {
            Self::Literal { .. } | Self::ScopeField { .. } => None,
            Self::Input { member, .. } => Some(member),
        }
    }
    pub fn source_field(&self) -> Option<&ApplicationFieldBindingLocus> {
        match self {
            Self::ScopeField { source, .. } => Some(source),
            Self::Literal { .. } | Self::Input { .. } => None,
        }
    }
    pub fn source_read(&self) -> Option<ApplicationOperationDecisionReadTarget> {
        self.source_field()
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

impl ApplicationOperationBodyProgram {
    /// Authority-free decoded description; fresh schema readmission validates it.
    pub fn from_untrusted_scope_fields(
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
    ) -> Self {
        Self {
            scope_fields,
            creation: None,
            requirements: Vec::new(),
        }
    }
    pub fn from_untrusted_with_creation(
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
        creation: Option<ApplicationOperationBodyCreation>,
    ) -> Self {
        Self {
            scope_fields,
            creation,
            requirements: Vec::new(),
        }
    }
    pub fn from_untrusted_with_requirements(
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
        creation: Option<ApplicationOperationBodyCreation>,
        requirements: Vec<super::ApplicationOperationBodyRequirementProgram>,
    ) -> Self {
        Self {
            scope_fields,
            creation,
            requirements,
        }
    }
    pub fn requirements(&self) -> &[super::ApplicationOperationBodyRequirementProgram] {
        &self.requirements
    }
    pub fn scope_fields(&self) -> &[ApplicationOperationBodyScopeField] {
        &self.scope_fields
    }
    pub fn creation(&self) -> Option<&ApplicationOperationBodyCreation> {
        self.creation.as_ref()
    }

    pub fn scope_entity(&self) -> Option<&str> {
        self.scope_fields.first().map(|step| step.field().entity())
    }
}
