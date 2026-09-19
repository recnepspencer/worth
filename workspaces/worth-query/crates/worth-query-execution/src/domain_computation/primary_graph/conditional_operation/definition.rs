use std::sync::Arc;

use super::canonical_identity::WorthQueryTemporalBindingIdentity;
use super::installation::ConditionalClockLease;

/// Immutable installed application meaning shared by operation instances.
/// Runtime lowerings, managed clocks, cursors, and retained work belong to each
/// instance; sharing this definition shares none of that mutable state.
pub(super) struct WorthQueryTemporalOperationDefinition<Binding, Reconstruction, Execution> {
    pub(super) binding_identity: Arc<WorthQueryTemporalBindingIdentity>,
    pub(super) installation_canonical_work:
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    pub(super) clock_lease: Arc<ConditionalClockLease>,
    pub(super) binding: Binding,
    pub(super) reconstruction: Reconstruction,
    pub(super) execution: Execution,
}
