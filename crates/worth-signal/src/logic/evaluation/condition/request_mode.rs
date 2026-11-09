use serde::{Deserialize, Serialize};

/// Request mode for one evaluation call.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvaluationRequestMode {
    /// Plan what is not current: requested targets whose upstream is settled
    /// are left out, and unchanged inputs validate a node clean.
    Default,
    /// Also re-execute requested targets that are already current, without
    /// validating them clean. Use it to demand a value, never to read one.
    ForceOnDemand,
}
