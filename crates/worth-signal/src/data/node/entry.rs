use serde::{Deserialize, Serialize};

mod access;
mod artifacts;
mod checkpoint;
mod definition;
mod layout;
mod state;
mod state_transitions;

#[cfg(test)]
mod tests;

pub(crate) use definition::NodeDefinitionData;

pub(crate) use layout::{
    node_hot_inline_size_bytes, node_warm_inline_size_bytes, NodeColdData, NodeHotData,
    NodeWarmData,
};
pub use state::NodeState;

/// Internal storage for a single signal node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeEntry {
    #[serde(flatten)]
    definition: NodeDefinitionData,
    #[serde(flatten)]
    hot: NodeHotData,
    #[serde(flatten)]
    warm: NodeWarmData,
    /// Cold diagnostics- and explanation-facing data kept off the hot path.
    #[serde(default)]
    cold: Option<Box<NodeColdData>>,
}
