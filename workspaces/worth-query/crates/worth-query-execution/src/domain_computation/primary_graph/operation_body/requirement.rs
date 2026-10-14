//! Descriptive result of the same installed precondition used by native admission.
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;

/// The current observation of one installed operation-body precondition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperationBodyRequirementStatus {
    /// The observed field satisfies the installed precondition.
    Met,
    /// The observed field does not satisfy the installed precondition.
    Unmet,
}

/// A descriptive finding for one installed body field; it grants no mutation authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryOperationBodyRequirementFinding {
    field: ApplicationFieldBindingLocus,
    status: WorthQueryOperationBodyRequirementStatus,
}

impl WorthQueryOperationBodyRequirementFinding {
    pub(in crate::domain_computation::primary_graph) fn new(
        field: ApplicationFieldBindingLocus,
        met: bool,
    ) -> Self {
        Self {
            field,
            status: if met {
                WorthQueryOperationBodyRequirementStatus::Met
            } else {
                WorthQueryOperationBodyRequirementStatus::Unmet
            },
        }
    }

    pub fn field(&self) -> &ApplicationFieldBindingLocus {
        &self.field
    }

    pub const fn status(&self) -> WorthQueryOperationBodyRequirementStatus {
        self.status
    }
}
