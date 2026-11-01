//! Canonical event and decision digests retain concrete ordered rosters.
use super::*;
use crate::history::data::CommitId;
use crate::identity::data::LineageId;
use crate::lineage::data::{
    LineageDecisionKind, LineageDecisionRecord, LineageEventKind, LineageEventRecord,
};

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "branch_id" => text(value, request)?,
                "lineage_event_ids" => {
                    vector::<u64>(value, request, |_| Ok(OwnerAllocation::default()))?
                }
                "lineage_events" => {
                    vector::<LineageEventRecord>(value, request, |value| row(value, request))?
                }
                "lineage_decision_log" => {
                    vector::<LineageDecisionRecord>(value, request, |value| row(value, request))?
                }
                "digest_basis" => digest(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn row(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "branch_id" => text(value, request)?,
                "commit" => super::semantic::receipt(value, request)?,
                "sources" | "targets" => {
                    vector::<LineageId>(value, request, |_| Ok(OwnerAllocation::default()))?
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn digest(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "branch_id" => text(value, request)?,
                "canonical_event_ids" => {
                    vector::<u64>(value, request, |_| Ok(OwnerAllocation::default()))?
                }
                "event_batch" => event_digest(value, request)?,
                "decision_log" => decision_digest(value, request)?,
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn event_digest(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "branch_id" => text(value, request)?,
                "canonical_event_ids" => {
                    vector::<u64>(value, request, |_| Ok(OwnerAllocation::default()))?
                }
                "canonical_commit_ids" => {
                    vector::<CommitId>(value, request, |_| Ok(OwnerAllocation::default()))?
                }
                "canonical_event_kinds" => {
                    vector::<LineageEventKind>(value, request, |_| Ok(OwnerAllocation::default()))?
                }
                "canonical_source_orderings" | "canonical_target_orderings" => {
                    orderings(value, request)?
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn decision_digest(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "branch_id" => text(value, request)?,
                "canonical_event_ids" => {
                    vector::<Option<u64>>(value, request, |_| Ok(OwnerAllocation::default()))?
                }
                "canonical_decision_kinds" => {
                    vector::<LineageDecisionKind>(value, request, |_| {
                        Ok(OwnerAllocation::default())
                    })?
                }
                "canonical_source_orderings" | "canonical_target_orderings" => {
                    orderings(value, request)?
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
fn orderings(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    vector::<Vec<LineageId>>(value, request, |value| {
        vector::<LineageId>(value, request, |_| Ok(OwnerAllocation::default()))
    })
}
