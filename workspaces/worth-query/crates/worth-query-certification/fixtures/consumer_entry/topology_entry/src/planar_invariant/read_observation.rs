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
    /// One completed reverse closure. Sizes describe its actual Native result;
    /// before/after are the platform's acquired-read observations.
    CompleteWalk {
        before: WorthQueryInvariantProjectionWork,
        after: WorthQueryInvariantProjectionWork,
        visited: usize,
        physical_relations: usize,
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
