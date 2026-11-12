use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::RuntimeBridge;

use crate::memory_workspace::WorthQueryWorkspaceError;
use crate::runtime::{WorthQueryIntentDeclaration, WorthQueryIntentExecution};

pub trait WorthQueryIntentAuthorityAdapter {
    fn execute_intent(
        &mut self,
        bridge: &RuntimeBridge,
        relational_runtime: Option<&RelationalRuntime>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryWorkspaceError>;
}
