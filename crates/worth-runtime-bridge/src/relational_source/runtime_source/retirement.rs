//! Relinquish this source's Native alias without destroying its registry payload.
use super::*;
use worth_execution::ExecutionRequest;
use worth_relational::facade::change_source::{
    RelationalRuntimeHandleAliasRetirementDenial, RetiredRelationalRuntimeHandleAlias,
};

/// Original source metadata after its local Native capability is retired.
/// Registrations issued to other owners keep their independent custody and
/// remain subject to Native's actual read-custody close fence.
pub struct RetiredRuntimeBridgeRelationalSource {
    _runtime: RetiredRelationalRuntimeHandleAlias,
    _observations: Option<observation_bindings::RelationalBridgeObservationBindings>,
    _heads: Option<branch_head_bindings::RelationalBridgeBranchHeadBindings>,
    _role: Arc<str>,
    _partition: Option<RelationalBridgePartitionBinding>,
    _selection: Arc<selected_commit_resolution::SourceSelectionWork>,
}

impl RuntimeBridgeRelationalSource {
    /// Validate the actual immutable Native pointee, without stopping service.
    pub fn preflight_alias_retirement(
        &self,
        owner: &RelationalRuntime,
    ) -> Result<(), RelationalRuntimeHandleAliasRetirementDenial> {
        self.runtime.validate_immutable_owner_alias(owner)
    }

    /// Consume only this source alias. Refusal returns the original source.
    /// The matching borrowed owner prevents the runtime share from being final.
    pub fn try_retire_alias_for_request(
        self,
        request: &ExecutionRequest<'_, '_>,
        owner: &RelationalRuntime,
    ) -> Result<
        RetiredRuntimeBridgeRelationalSource,
        (Self, RelationalRuntimeHandleAliasRetirementDenial),
    > {
        let Self {
            runtime,
            observation_bindings,
            branch_head_bindings,
            graph_role,
            partition,
            selection_work,
        } = self;
        match runtime.try_retire_immutable_alias(request, owner) {
            Ok(runtime) => Ok(RetiredRuntimeBridgeRelationalSource {
                _runtime: runtime,
                // Every outstanding registration owns its issuing registry Arc.
                // Nonfinal local release destroys no row. A unique registry has
                // no issued registrations; retain its original storage anyway.
                _observations: Arc::into_inner(observation_bindings),
                _heads: Arc::into_inner(branch_head_bindings),
                _role: graph_role,
                _partition: partition,
                _selection: selection_work,
            }),
            Err((runtime, cause)) => Err((
                Self {
                    runtime,
                    observation_bindings,
                    branch_head_bindings,
                    graph_role,
                    partition,
                    selection_work,
                },
                cause,
            )),
        }
    }
}

impl RetiredRuntimeBridgeRelationalSource {
    /// Explicit unbudgeted reclamation, after local capability retirement.
    pub fn reclaim(self) {
        drop(self);
    }
}
