//! Ordered, portable native values for one operation's scope-bound body.

use worth_foundational::facade::AspectValue;

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
}

impl ApplicationOperationBodyScopeField {
    pub fn field(&self) -> &ApplicationFieldBindingLocus {
        match self {
            Self::Expect { field, .. } | Self::Assign { field, .. } => field,
        }
    }

    pub fn binding(&self) -> &WorthQueryPortableTypeIdentity {
        match self {
            Self::Expect { binding, .. } | Self::Assign { binding, .. } => binding,
        }
    }

    pub fn value(&self) -> &AspectValue {
        match self {
            Self::Expect { value, .. } | Self::Assign { value, .. } => value,
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
        let Self::Assign { field, .. } = self else {
            return None;
        };
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
}

impl ApplicationOperationBodyProgram {
    /// Authority-free decoded description; fresh schema readmission validates it.
    pub fn from_untrusted_scope_fields(
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
    ) -> Self {
        Self { scope_fields }
    }
    pub fn scope_fields(&self) -> &[ApplicationOperationBodyScopeField] {
        &self.scope_fields
    }

    pub fn scope_entity(&self) -> Option<&str> {
        self.scope_fields.first().map(|step| step.field().entity())
    }
}
