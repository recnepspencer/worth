//! Scope observation and assignment steps; portable data grants no authority.
use crate::application_schema::{
    ApplicationFieldBindingLocus, ApplicationOperationDecisionReadTarget,
    ApplicationOperationProgramTarget,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::facade::{AspectValue, ScalarAspectType};
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationOperationBodyScopeField {
    ExpectInput {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        scalar_family: ScalarAspectType,
        member: String,
    },
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
    AssignValue {
        field: ApplicationFieldBindingLocus,
        binding: WorthQueryPortableTypeIdentity,
        value: String,
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
            Self::ExpectInput { field, .. }
            | Self::Expect { field, .. }
            | Self::Assign { field, .. }
            | Self::AssignInput { field, .. }
            | Self::AssignValue { field, .. }
            | Self::SetOptionalInput { field, .. }
            | Self::AssignOptionalInput { field, .. }
            | Self::ClearOptional { field, .. }
            | Self::RequirePresent { field, .. } => field,
        }
    }

    pub fn binding(&self) -> &WorthQueryPortableTypeIdentity {
        match self {
            Self::ExpectInput { binding, .. }
            | Self::Expect { binding, .. }
            | Self::Assign { binding, .. }
            | Self::AssignInput { binding, .. }
            | Self::AssignValue { binding, .. }
            | Self::SetOptionalInput { binding, .. }
            | Self::AssignOptionalInput { binding, .. }
            | Self::ClearOptional { binding, .. }
            | Self::RequirePresent { binding, .. } => binding,
        }
    }

    pub fn literal_value(&self) -> Option<&AspectValue> {
        match self {
            Self::Expect { value, .. } | Self::Assign { value, .. } => Some(value),
            Self::ExpectInput { .. }
            | Self::AssignInput { .. }
            | Self::AssignValue { .. }
            | Self::SetOptionalInput { .. }
            | Self::AssignOptionalInput { .. }
            | Self::ClearOptional { .. }
            | Self::RequirePresent { .. } => None,
        }
    }

    pub fn scalar_family(&self) -> ScalarAspectType {
        match self {
            Self::Expect { value, .. } | Self::Assign { value, .. } => value.value_family(),
            Self::AssignValue { .. } => ScalarAspectType::UInt64,
            Self::ExpectInput { scalar_family, .. }
            | Self::AssignInput { scalar_family, .. }
            | Self::SetOptionalInput { scalar_family, .. }
            | Self::AssignOptionalInput { scalar_family, .. }
            | Self::ClearOptional { scalar_family, .. }
            | Self::RequirePresent { scalar_family, .. } => *scalar_family,
        }
    }
    pub fn input_member(&self) -> Option<&str> {
        match self {
            Self::ExpectInput { member, .. }
            | Self::AssignInput { member, .. }
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
        if matches!(
            self,
            Self::Expect { .. } | Self::ExpectInput { .. } | Self::RequirePresent { .. }
        ) {
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
