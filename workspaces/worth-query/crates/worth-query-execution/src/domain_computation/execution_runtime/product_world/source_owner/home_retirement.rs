//! This actual source still owns operational Native and Bridge capabilities.
use super::WorthQueryRelationalSourceOwner;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
};

impl WorthQueryRelationalSourceOwner {
    pub(in crate::domain_computation) fn advance_home_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryOperationHomeRetirementProgress,
        Denial,
    > {
        request.consult().map_err(Denial::Execution)?;
        Err(Denial::Unsupported(Owner::RelationalSource))
    }

    pub(in crate::domain_computation) fn preflight_home_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        request.consult().map_err(Denial::Execution)?;
        // runtime, source, bridge_head, invalidation_owner and canonical_subscription
        // retain their actual operational owners. No consuming owner port exists
        // yet; descriptions or handler completion cannot release those owners.
        Err(Denial::Unsupported(Owner::RelationalSource))
    }
}
