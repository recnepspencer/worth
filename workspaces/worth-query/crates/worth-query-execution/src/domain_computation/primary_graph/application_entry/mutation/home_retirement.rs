//! Handler-local observations never certify Native or physical Home close.

#[derive(Debug)]
pub enum WorthQueryOperationHomeRetirementDenial {
    Busy,
    Unsupported,
    Execution(worth_execution::WorkCeilingDenial),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WorthQueryOperationHomeRetirementProgress {
    Advanced,
    Complete,
}
#[cfg(test)]
#[path = "home_retirement_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
