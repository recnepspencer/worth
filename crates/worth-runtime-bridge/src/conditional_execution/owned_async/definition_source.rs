//! Authored definition meaning; a current Native owner still issues all resources.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::conditional_execution) struct OwnedAsyncDefinitionSource {
    pub(in crate::conditional_execution) node: worth_signal::facade::NodeId,
    pub(in crate::conditional_execution) description: super::BridgeOwnedAsyncDeclarationDescription,
}
