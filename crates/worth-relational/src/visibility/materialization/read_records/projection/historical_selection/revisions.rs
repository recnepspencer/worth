//! Match original observations only while the final revision still equals restoration.
use super::*;
use crate::execution::request_stage::{checkpoint, optional_stage};
use crate::identity::data::EntityId;
use crate::storage::data::RelationalFieldRevision;
use worth_foundational::facade::{AspectFieldLocator, AspectKey};

impl HistoricalSelectionStage {
    fn member(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &VisibilityProjectionView,
        entity: EntityId,
    ) -> Result<bool, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        if current.runtime_identity != self.restored.runtime_identity || !current.is_exact_basis() {
            return Err(Denial::ForeignOwner);
        }
        if current.version_id() < self.restored.version_id()
            || current
                .selected_schema_authority()
                .map(|s| s.authority_digest())
                != self
                    .restored
                    .selected_schema_authority()
                    .map(|s| s.authority_digest())
        {
            return Ok(false);
        }
        optional_stage(
            execution,
            |context, _| {
                checkpoint(
                    context,
                    self.graph()
                        .selected_entities()
                        .len()
                        .checked_ilog2()
                        .unwrap_or(0) as u64
                        + 1,
                )
                .map_err(crate::execution::denial::kernel_stop)?;
                let Some(kind) = self.graph().selected_entity_kind(entity) else {
                    return Ok(false);
                };
                for view in [&self.selected, &self.restored, current] {
                    checkpoint(
                        context,
                        view.exact_entity_state_read_work_bound().unwrap_or(0),
                    )
                    .map_err(crate::execution::denial::kernel_stop)?;
                    if view
                        .read_exact_entity_state(execution, entity, |metadata, _| metadata.kind_id)
                        .map_err(|cause| match cause {
                            super::RelationalBorrowedRecordReadDenial::Execution(cause) => cause,
                            _ => Cause::NestedStopped,
                        })?
                        != Some(kind)
                    {
                        return Ok(false);
                    }
                }
                Ok(true)
            },
            |_| 0,
        )
        .map_err(Denial::Execution)?
        .map_err(Denial::Execution)
    }
}

/// A matching revision ends the walk; only an authentic restoration baseline
/// permits comparing the unchanged original observation at an earlier link.
enum RevisionComparison {
    Current,
    Prior,
    Changed,
}

impl RelationalHistoricalSelectionRead {
    /// Match an immutable producer field observation through every restoration
    /// it needs. A changed intermediate revision, including ABA, ends the proof.
    pub fn matches_field_revision(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &VisibilityProjectionView,
        entity: EntityId,
        locator: &AspectFieldLocator,
        expected: RelationalFieldRevision,
    ) -> Result<bool, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        let mut current = current;
        for stage in self.stages.iter().rev() {
            if !stage.member(execution, current, entity)? {
                return Ok(false);
            }
            let comparison = optional_stage(
                execution,
                |context, _| {
                    let mut read = |view: &VisibilityProjectionView| {
                        view.entity_field_revision_admitted(execution, entity, locator, |work| {
                            checkpoint(context, work)
                        })?
                        .map_err(crate::execution::denial::kernel_stop)
                    };
                    let actual = read(current)?;
                    if actual.is_none() || actual != read(&stage.restored)? {
                        return Ok(RevisionComparison::Changed);
                    }
                    Ok(if read(&stage.selected)? == Some(expected) {
                        RevisionComparison::Current
                    } else {
                        RevisionComparison::Prior
                    })
                },
                |_| 0,
            )
            .map_err(Denial::Execution)?
            .map_err(Denial::Execution)?;
            match comparison {
                RevisionComparison::Current => return Ok(true),
                RevisionComparison::Changed => return Ok(false),
                RevisionComparison::Prior => current = &stage.selected,
            }
        }
        Ok(false)
    }

    /// Output witnesses use the same prefix of authentic restoration links.
    pub fn matches_aspect_revision(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &VisibilityProjectionView,
        entity: EntityId,
        aspect: &AspectKey,
        expected: Option<u64>,
    ) -> Result<bool, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        let mut current = current;
        for stage in self.stages.iter().rev() {
            if !stage.member(execution, current, entity)? {
                return Ok(false);
            }
            let comparison = optional_stage(
                execution,
                |context, _| {
                    let mut read = |view: &VisibilityProjectionView| {
                        checkpoint(context, 128u64.saturating_add(aspect.as_str().len() as u64))
                            .map_err(|stop| {
                                Denial::Execution(crate::execution::denial::kernel_stop(stop))
                            })?;
                        view.read_entity_aspect_version(execution, entity, aspect)
                            .map_err(borrowed)
                    };
                    let actual = read(current)?;
                    if actual.is_none() || actual != read(&stage.restored)? {
                        return Ok(RevisionComparison::Changed);
                    }
                    Ok(if read(&stage.selected)? == Some(expected) {
                        RevisionComparison::Current
                    } else {
                        RevisionComparison::Prior
                    })
                },
                |_| 0,
            )
            .map_err(Denial::Execution)??;
            match comparison {
                RevisionComparison::Current => return Ok(true),
                RevisionComparison::Changed => return Ok(false),
                RevisionComparison::Prior => current = &stage.selected,
            }
        }
        Ok(false)
    }
}
