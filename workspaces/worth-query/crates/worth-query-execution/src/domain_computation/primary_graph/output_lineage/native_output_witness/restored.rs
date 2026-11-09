//! A transported output compares its witness through the selection's read evidence.
use super::*;
use worth_relational::facade::runtime::{
    RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial,
    VisibilityProjectionView,
};

impl SealedNativeOutputWitness {
    /// Like `unchanged_in`, except an aspect whose revision moved is held
    /// only when the evidence proves the original revision at the selected
    /// step and no change since the restoring commit. Presence and
    /// retirement checks are the ordinary ones. O(roles + aspects).
    pub(in crate::domain_computation::primary_graph) fn unchanged_since_restoration(
        &self,
        runtime: &RelationalRuntime,
        selected: &SnapshotHandle,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        current: &VisibilityProjectionView,
        evidence: &RelationalHistoricalSelectionRead,
    ) -> Result<Result<bool, RelationalHistoricalSelectionReadDenial>, CompanionPreflightStop> {
        let read = runtime.read_truth();
        for role in &self.roles {
            if !role.remains_present(runtime, selected, execution)? {
                return Ok(Ok(false));
            }
            if role.posture == WorthQueryApplicationOutputPosture::Retire {
                continue;
            }
            let entity = role.entity.expect("present role has an entity");
            for aspect in &self.aspects[role.first_aspect..role.end_aspect] {
                let actual = read
                    .read_exact_snapshot_entity_aspect_version(
                        execution,
                        selected,
                        entity,
                        &aspect.aspect,
                    )
                    .map_err(CompanionPreflightStop::Execution)?;
                if actual == aspect.revision {
                    continue;
                }
                let Some(expected) = aspect.revision else {
                    return Ok(Ok(false));
                };
                match evidence.matches_aspect_revision(
                    execution,
                    current,
                    entity,
                    &aspect.aspect,
                    expected,
                ) {
                    Ok(true) => {}
                    other => return Ok(other),
                }
            }
        }
        Ok(Ok(true))
    }
}
