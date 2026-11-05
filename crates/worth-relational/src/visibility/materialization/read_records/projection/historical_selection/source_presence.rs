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

impl RelationalHistoricalSelectionRead {
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
