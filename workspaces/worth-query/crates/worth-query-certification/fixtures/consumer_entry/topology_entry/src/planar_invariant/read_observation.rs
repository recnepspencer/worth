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
    /// One complete orbit. Counts describe its business walk; private Native
    /// identity metadata preparation is outside these application observations.
    CompleteOrbit {
        before: WorthQueryInvariantProjectionWork,
        after: WorthQueryInvariantProjectionWork,
        visited: usize,
        physical_relations: usize,
        identity: [u8; 32],
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
