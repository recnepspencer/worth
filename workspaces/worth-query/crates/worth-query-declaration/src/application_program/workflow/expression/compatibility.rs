//! Readmission of the version-1 condition format.
//!
//! A version-1 condition named one installed Bool query and took its result
//! as the decision. It migrates to the expression `condition` reading that
//! query's result as the operand `condition`: the same query, the same source
//! proof, and the same decision. The old format has no interpreter of its
//! own; it exists only as data this readmits.

use super::condition::{ApplicationWorkflowCondition, ApplicationWorkflowConditionDenial};
use super::operands::{ApplicationWorkflowConditionOperand, ApplicationWorkflowConditionQuery};

/// The operand, and the whole expression, a migrated condition reads.
pub const MIGRATED_WORKFLOW_CONDITION_OPERAND: &str = "condition";

impl ApplicationWorkflowCondition {
    /// Migrates a version-1 condition over `query`. Admission requires the
    /// expression to be Bool, so a query of any other result type denies.
    pub fn migrated(
        query: ApplicationWorkflowConditionQuery,
    ) -> Result<Self, ApplicationWorkflowConditionDenial> {
        Self::parse(
            MIGRATED_WORKFLOW_CONDITION_OPERAND,
            vec![ApplicationWorkflowConditionOperand::new(
                MIGRATED_WORKFLOW_CONDITION_OPERAND,
                query,
            )],
        )
    }
}
