//! Source presence is a fact of this exact committed restoration, never permission.
use super::*;
use crate::identity::data::{EntityId, KindId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalHistoricalSourcePresence {
    LiveInDomain,
    NotLive,
    KindMismatch { actual: KindId },
    OutsideDomain,
}

#[derive(Debug)]
pub enum RelationalHistoricalSourceReadDenial {
    Selection(RelationalHistoricalSelectionReadDenial),
    Ownership(Box<crate::branch::RelationalModelOwnershipDenial>),
}

impl RelationalHistoricalSelectionRead {
    pub(crate) fn contains_selected_entity(&self, entity: EntityId, kind: KindId) -> bool {
        self.latest().graph().selected_entity_kind(entity) == Some(kind)
    }

    /// Membership and liveness at a progressed exact candidate. Only declared
    /// ownership and current publication custody can connect a new source to
    /// this sealed selected boundary; sharing an outside document is insufficient.
    pub fn source_presence_at(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        native: &crate::runtime::RelationalRuntime,
        basis: &crate::branch::AdmittedRelationalBranchBasis,
        ownership: &crate::branch::RelationalModelOwnership,
        entity: EntityId,
        expected_kind: KindId,
    ) -> Result<RelationalHistoricalSourcePresence, RelationalHistoricalSourceReadDenial> {
        if basis.identity().runtime_instance_id() != native.runtime_instance_id()
            || !self.latest().restored.is_from_runtime(native)
        {
            return Err(RelationalHistoricalSourceReadDenial::Selection(
                RelationalHistoricalSelectionReadDenial::ForeignOwner,
            ));
        }
        if basis.descriptor().branch_id() != self.latest().restored.basis.branch_id()
            || basis.observation().version_id() < self.latest().restored.version_id()
        {
            return Err(RelationalHistoricalSourceReadDenial::Selection(
                RelationalHistoricalSelectionReadDenial::SelectedBasisMismatch {
                    commit: self.selection_commit(),
                },
            ));
        }
        let work = self
            .latest()
            .graph()
            .selected_entities()
            .len()
            .checked_ilog2()
            .unwrap_or(0) as u64
            + 1;
        let observed = native
            .observe_model_source_boundary(
                execution,
                basis,
                ownership,
                entity,
                expected_kind,
                work,
                &mut |entity, kind| self.contains_selected_entity(entity, kind),
            )
            .map_err(|cause| RelationalHistoricalSourceReadDenial::Ownership(Box::new(cause)))?;
        use RelationalHistoricalSourcePresence as Presence;
        Ok(match observed {
            None => Presence::NotLive,
            Some((kind, _, _)) if kind != expected_kind => Presence::KindMismatch { actual: kind },
            Some((_, Some(_), true)) => Presence::LiveInDomain,
            _ => Presence::OutsideDomain,
        })
    }
    /// Reads only this capsule's own D. Query must establish current C
    /// authorization before consuming an identity/presence refusal here.
    pub fn source_presence(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        entity: EntityId,
        expected_kind: KindId,
    ) -> Result<RelationalHistoricalSourcePresence, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        use RelationalHistoricalSourcePresence as Presence;
        let stage = self.latest();
        let actual = stage
            .restored
            .read_exact_entity_state(execution, entity, |metadata, _| metadata.kind_id)
            .map_err(borrowed)?;
        let Some(actual) = actual else {
            return Ok(Presence::NotLive);
        };
        if actual != expected_kind {
            return Ok(Presence::KindMismatch { actual });
        }
        execution
            .run_native_with_checkpoint(|context| {
                crate::execution::request_stage::checkpoint(
                    context,
                    stage
                        .graph()
                        .selected_entities()
                        .len()
                        .checked_ilog2()
                        .unwrap_or(0) as u64
                        + 1,
                )
            })
            .map_err(Denial::Request)?
            .0
            .map_err(|stop| Denial::Request(worth_execution::WorkCeilingDenial::Stopped(stop)))?;
        Ok(
            if stage.graph().selected_entity_kind(entity) == Some(expected_kind) {
                Presence::LiveInDomain
            } else {
                Presence::OutsideDomain
            },
        )
    }
}
