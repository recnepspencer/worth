//! Definition-level actions apply to live frontiers, not synthetic control nodes.
use super::ApplicationWorkflowOperationRef;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationWorkflowGlobalAction {
    /// Run the declared local business body and end the instance atomically.
    Terminate(ApplicationWorkflowOperationRef),
}

impl ApplicationWorkflowGlobalAction {
    pub const fn operation(&self) -> &ApplicationWorkflowOperationRef {
        match self {
            Self::Terminate(operation) => operation,
        }
    }
}
