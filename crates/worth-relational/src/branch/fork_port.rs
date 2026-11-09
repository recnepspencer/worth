use std::sync::Arc;

use crate::durability::medium::RelationalDurableMediumBinding;
use crate::runtime::{RelationalForkOwnerBinding, RelationalRuntimeOwnerBinding};

/// Cloneable branch-fork service bound to one live relational runtime owner.
/// It publishes each fork to the runtime's durable medium, when the runtime
/// owns one, before the target exists.
#[derive(Debug, Clone)]
pub struct RelationalForkPort {
    pub(super) runtime_instance_id: u64,
    pub(super) lifecycle: RelationalRuntimeOwnerBinding,
    pub(super) owner: RelationalForkOwnerBinding,
    pub(super) medium: Option<Arc<RelationalDurableMediumBinding>>,
    pub(super) publication: crate::runtime::RelationalRuntimePublicationBinding,
}

impl RelationalForkPort {
    /// Observe the actual canonical/transition originals before a composing
    /// owner's creation intent. Both streams are fixed by the same existing
    /// position reservation; the returned coordinates grant no fork authority.
    pub fn observe_original_frontiers(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            crate::publication::patch::data::PatchStreamPosition,
            super::RelationalBranchTransitionSequence,
        ),
        super::RelationalBranchTransitionStop<super::RelationalForkDenial>,
    > {
        let _operation =
            self.lifecycle
                .admit()
                .ok_or(super::RelationalBranchTransitionStop::Denied(
                    super::RelationalForkDenial::OwnerUnavailable,
                ))?;
        self.owner
            .observe_original_frontiers(execution)
            .map_err(super::RelationalBranchTransitionStop::RequestStopped)
    }

    pub(crate) fn new(
        runtime_instance_id: u64,
        lifecycle: RelationalRuntimeOwnerBinding,
        owner: RelationalForkOwnerBinding,
        medium: Option<Arc<RelationalDurableMediumBinding>>,
        publication: crate::runtime::RelationalRuntimePublicationBinding,
    ) -> Self {
        Self {
            runtime_instance_id,
            lifecycle,
            owner,
            medium,
            publication,
        }
    }
}
