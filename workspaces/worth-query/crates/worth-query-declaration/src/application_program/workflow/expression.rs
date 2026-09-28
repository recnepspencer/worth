//! Expression-backed workflow conditions over declared query results.

mod compatibility;
mod condition;
mod operands;

pub use compatibility::MIGRATED_WORKFLOW_CONDITION_OPERAND;
pub use condition::{ApplicationWorkflowCondition, ApplicationWorkflowConditionDenial};
pub use operands::{
    ApplicationExpressionOperandValue, ApplicationWorkflowConditionOperand,
    ApplicationWorkflowConditionOperands, ApplicationWorkflowConditionQuery,
};
