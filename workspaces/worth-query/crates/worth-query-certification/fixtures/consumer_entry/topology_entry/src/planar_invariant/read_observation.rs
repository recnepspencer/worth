//! Optional caller-owned diagnostics over the platform's shared read value.
use super::*;

pub type PlanarInvariantReadObserver = Arc<
    dyn Fn(PlanarInvariantReadEvent)
        + Send
        + Sync
        + std::panic::RefUnwindSafe
        + std::panic::UnwindSafe,
>;

/// Raw observations from one serial rule invocation. No event grants access or
/// determines the verdict. Preparation/evaluation events also describe error
/// returns; a completed evaluation may return either Pass or Violation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanarInvariantReadEvent {
    Touched {
        entity: EntityId,
    },
    /// Two completed forward reads for this genuine native root.
    ForwardHops {
        root: EntityId,
        before: WorthQueryInvariantProjectionWork,
        after: WorthQueryInvariantProjectionWork,
    },
    Prepared {
        reads: WorthQueryInvariantProjectionWork,
        succeeded: bool,
    },
    Evaluated {
        reads: WorthQueryInvariantProjectionWork,
        returned_verdict: bool,
    },
}
