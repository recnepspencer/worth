//! Match original observations only while the final revision still equals restoration.
use super::*;
use crate::execution::request_stage::{checkpoint, optional_stage};
use crate::identity::data::{EntityId, KindId, VersionId};
use crate::storage::data::RelationalFieldRevision;
use crate::visibility::materialization::read_records::projection::{
    RelationalAdjacencyDirection, RelationalAdjacencyVisit,
};
use worth_foundational::facade::{AspectFieldLocator, AspectKey};

impl HistoricalSelectionStage {
    /// A graph selection may leave external incoming edges or a transferred
    /// member's ordinary outgoing edges current. Both exact lists must be
    /// wholly covered by its canonical selected relation roster; equal list
    /// counts then establish complete restoration without copying records.
    fn restored_adjacency_complete(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        anchor: EntityId,
        kind: KindId,
        direction: RelationalAdjacencyDirection,
    ) -> Result<bool, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        optional_stage(
            execution,
            |context, _| {
                let roster = self.graph().selected_relations();
                let mut complete = true;
                let mut counts = [0u64; 2];
                for (index, view) in [&self.selected, &self.restored].into_iter().enumerate() {
                    view.try_visit_adjacency_ids(execution, anchor, kind, direction, |visit| {
                        match visit {
                            RelationalAdjacencyVisit::Prepare { work, .. } => {
                                checkpoint(context, work)
                                    .map_err(crate::execution::denial::kernel_stop)?;
                            }
                            RelationalAdjacencyVisit::Relation(id) => {
                                checkpoint(
                                    context,
                                    roster.len().checked_ilog2().unwrap_or(0) as u64 + 1,
                                )
                                .map_err(crate::execution::denial::kernel_stop)?;
                                let edge = roster
                                    .binary_search_by_key(&id, |edge| edge.relation_id)
                                    .ok()
                                    .map(|index| &roster[index]);
                                complete &= edge.is_some_and(|edge| {
                                    edge.kind_id == kind
                                        && match direction {
                                            RelationalAdjacencyDirection::Outgoing => {
                                                edge.source == anchor
                                            }
                                            RelationalAdjacencyDirection::Incoming => {
                                                edge.target == anchor
                                            }
                                        }
                                });
                                counts[index] += 1;
                            }
                            RelationalAdjacencyVisit::List | RelationalAdjacencyVisit::Acquired => {
                                checkpoint(context, 1)
                                    .map_err(crate::execution::denial::kernel_stop)?;
                            }
                        }
                        Ok::<_, Cause>(())
                    })??;
                }
                Ok(complete && counts[0] == counts[1])
            },
            |_| 0,
        )
        .map_err(Denial::Execution)?
        .map_err(Denial::Execution)
    }

    fn member(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &VisibilityProjectionView,
        entity: EntityId,
    ) -> Result<bool, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        if !current.shares_runtime_with(&self.restored) || !current.is_exact_basis() {
            return Err(Denial::ForeignOwner);
        }
        if current.version_id() < self.restored.version_id()
            || current
                .selected_schema()
                .map(|schema| schema.authority_digest)
                != self
                    .restored
                    .selected_schema()
                    .map(|schema| schema.authority_digest)
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
    /// Match an original adjacency revision only through complete, authentic
    /// relation restoration. Later structural edits, including ABA, end proof.
    pub fn matches_adjacency_revision(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        current: &VisibilityProjectionView,
        anchor: EntityId,
        kind: KindId,
        direction: RelationalAdjacencyDirection,
        expected: Option<VersionId>,
    ) -> Result<bool, RelationalHistoricalSelectionReadDenial> {
        use RelationalHistoricalSelectionReadDenial as Denial;
        let mut current = current;
        for stage in self.stages.iter().rev() {
            if !stage.member(execution, current, anchor)? {
                return Ok(false);
            }
            let comparison = optional_stage(
                execution,
                |context, _| {
                    let mut read = |view: &VisibilityProjectionView| {
                        view.read_exact_adjacency_structural_revision_admitted(
                            execution,
                            anchor,
                            kind,
                            direction,
                            |work| checkpoint(context, work),
                        )?
                        .map_err(|cause| match cause {
                            RelationalBorrowedRecordReadDenial::Execution(cause) => cause,
                            RelationalBorrowedRecordReadDenial::ExactBasisRequired => {
                                Cause::NestedStopped
                            }
                        })
                    };
                    if read(current)? != read(&stage.restored)? {
                        return Ok(RevisionComparison::Changed);
                    }
                    Ok(if read(&stage.selected)? == expected {
                        RevisionComparison::Current
                    } else {
                        RevisionComparison::Prior
                    })
                },
                |_| 0,
            )
            .map_err(Denial::Execution)?
            .map_err(Denial::Execution)?;
            if matches!(comparison, RevisionComparison::Changed)
                || !stage.restored_adjacency_complete(execution, anchor, kind, direction)?
            {
                return Ok(false);
            }
            match comparison {
                RevisionComparison::Current => return Ok(true),
                RevisionComparison::Prior => current = &stage.selected,
                RevisionComparison::Changed => unreachable!("changed revision declined above"),
            }
        }
        Ok(false)
    }

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
