//! Ordered, portable native values for one operation's scope-bound body.

use worth_foundational::facade::{AspectValue, ScalarAspectType};

pub use super::created_field::ApplicationOperationBodyCreatedField;
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
    AssignOptionalInput {
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
            | Self::AssignOptionalInput { field, .. }
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
            | Self::AssignOptionalInput { binding, .. }
            | Self::ClearOptional { binding, .. }
            | Self::RequirePresent { binding, .. } => binding,
        }
    }

    pub fn literal_value(&self) -> Option<&AspectValue> {
        match self {
            Self::Expect { value, .. } | Self::Assign { value, .. } => Some(value),
            Self::AssignInput { .. }
            | Self::SetOptionalInput { .. }
            | Self::AssignOptionalInput { .. }
            | Self::ClearOptional { .. }
            | Self::RequirePresent { .. } => None,
        }
    }

    pub fn scalar_family(&self) -> ScalarAspectType {
        match self {
            Self::Expect { value, .. } | Self::Assign { value, .. } => value.value_family(),
            Self::AssignInput { scalar_family, .. }
            | Self::SetOptionalInput { scalar_family, .. }
            | Self::AssignOptionalInput { scalar_family, .. }
            | Self::ClearOptional { scalar_family, .. }
            | Self::RequirePresent { scalar_family, .. } => *scalar_family,
        }
    }
    pub fn input_member(&self) -> Option<&str> {
        match self {
            Self::AssignInput { member, .. }
            | Self::SetOptionalInput { member, .. }
            | Self::AssignOptionalInput { member, .. } => Some(member),
            _ => None,
        }
    }
    pub fn input_is_optional(&self) -> bool {
        matches!(self, Self::AssignOptionalInput { .. })
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
    pub(super) explicit_scope: Option<String>,
    pub(super) scope_fields: Vec<ApplicationOperationBodyScopeField>,
    pub(super) creation: Option<ApplicationOperationBodyCreation>,
    pub(super) requirements: Vec<super::ApplicationOperationBodyRequirementProgram>,
    pub(super) outputs: Vec<super::ApplicationOperationBodyOutput>,
}

/// One typed, candidate-local creation site in the admitted scope's context.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyCreation {
    pub(super) entity: String,
    pub(super) key: String,
    pub(super) fields: Vec<ApplicationOperationBodyCreatedField>,
    pub(super) links: Vec<super::ApplicationOperationBodyCreationLink>,
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
            links: Vec::new(),
        }
    }
    pub fn entity(&self) -> &str {
        &self.entity
    }
    pub fn with_untrusted_links(
        mut self,
        links: Vec<super::ApplicationOperationBodyCreationLink>,
    ) -> Self {
        self.links = links;
        self
    }
    pub fn links(&self) -> &[super::ApplicationOperationBodyCreationLink] {
        &self.links
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn fields(&self) -> &[ApplicationOperationBodyCreatedField] {
        &self.fields
    }
}

impl ApplicationOperationBodyProgram {
    /// Authority-free decoded description; fresh schema readmission validates it.
    pub fn from_untrusted_scope_fields(
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
    ) -> Self {
        Self {
            explicit_scope: None,
            scope_fields,
            creation: None,
            requirements: Vec::new(),
            outputs: Vec::new(),
        }
    }
    pub fn from_untrusted_with_creation(
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
        creation: Option<ApplicationOperationBodyCreation>,
    ) -> Self {
        Self {
            explicit_scope: None,
            scope_fields,
            creation,
            requirements: Vec::new(),
            outputs: Vec::new(),
        }
    }
    pub fn from_untrusted_with_requirements(
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
        creation: Option<ApplicationOperationBodyCreation>,
        requirements: Vec<super::ApplicationOperationBodyRequirementProgram>,
    ) -> Self {
        Self {
            explicit_scope: None,
            scope_fields,
            creation,
            requirements,
            outputs: Vec::new(),
        }
    }
    /// Authority-free scope-bound description. Fresh readmission checks the scope
    /// and its exact decision membership. Redundant field-derived scope normalizes.
    pub fn from_untrusted_in_scope(
        scope: String,
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
        creation: Option<ApplicationOperationBodyCreation>,
        requirements: Vec<super::ApplicationOperationBodyRequirementProgram>,
    ) -> Self {
        let explicit_scope = (scope_fields.first().map(|step| step.field().entity())
            != Some(scope.as_str()))
        .then_some(scope);
        Self {
            explicit_scope,
            scope_fields,
            creation,
            requirements,
            outputs: Vec::new(),
        }
    }
    pub fn explicit_scope_entity(&self) -> Option<&str> {
        self.explicit_scope.as_deref()
    }
    /// Untrusted output descriptions must be admitted with their mutation contract.
    pub fn with_untrusted_outputs(
        mut self,
        outputs: Vec<super::ApplicationOperationBodyOutput>,
    ) -> Self {
        self.outputs = outputs;
        self
    }
    pub fn outputs(&self) -> &[super::ApplicationOperationBodyOutput] {
        &self.outputs
    }
    /// Creation changes no existing scope field; guards and requirements may read it.
    pub fn is_creation_only(&self) -> bool {
        self.creation.is_some()
            && self
                .scope_fields
                .iter()
                .all(|step| step.write_target().is_none())
    }
    pub fn requirements(&self) -> &[super::ApplicationOperationBodyRequirementProgram] {
        &self.requirements
    }
    pub fn has_optional_inputs(&self) -> bool {
        self.scope_fields
            .iter()
            .any(ApplicationOperationBodyScopeField::input_is_optional)
            || self
                .creation
                .iter()
                .flat_map(|creation| creation.fields())
                .any(ApplicationOperationBodyCreatedField::input_is_optional)
    }
    pub fn scope_fields(&self) -> &[ApplicationOperationBodyScopeField] {
        &self.scope_fields
    }
    pub fn creation(&self) -> Option<&ApplicationOperationBodyCreation> {
        self.creation.as_ref()
    }

    pub fn scope_entity(&self) -> Option<&str> {
        self.explicit_scope
            .as_deref()
            .or_else(|| self.scope_fields.first().map(|step| step.field().entity()))
    }
}
