use serde::{Deserialize, Serialize};

/// Three-state invalidation for a signal node.
///
/// This is the core reactive primitive:
/// - `Clean`: value is current, no recomputation needed
/// - `MaybeStale`: a transitive dependency changed - check before using
/// - `Dirty`: a direct dependency changed - must recompute
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    /// No change has reached this node's own inputs. Invalidation advances
    /// one level per evaluation, so a clean node can still sit below an
    /// unsettled ancestor; it is current only when the planner finds no
    /// unsettled path above it.
    Clean,
    /// A dependency's dependency changed. May or may not affect this node.
    /// Requires walking upstream to determine if recomputation is needed.
    MaybeStale,
    /// A direct dependency changed. This node MUST recompute.
    Dirty,
}
