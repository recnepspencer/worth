//! Quote the escaping cascade owners and their complete decoded publication trees.
use super::*;
use crate::history::data::{
    CanonicalAuthoredRestoration, CanonicalAuthoredRetirement, CanonicalProducerRetirement,
    CanonicalRequiredOutputOrigin,
};
use crate::identity::data::{EntityId, RelationId};
use std::sync::Arc;

pub(super) fn producer(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    cascade::<CanonicalProducerRetirement>(value, request)
}
pub(super) fn authored(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    cascade::<CanonicalAuthoredRetirement>(value, request)
}
pub(super) fn restoration(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    cascade::<CanonicalAuthoredRestoration>(value, request)
}
fn cascade<T>(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    optional(value, |value| {
        let mut bytes = shared_root::<T>()?;
        value.fields(request, |field, value| {
            let nested = match field {
                "publications" => {
                    vector::<Arc<CanonicalRequiredOutputOrigin>>(value, request, |origin| {
                        super::required_output::origin(origin, request)
                    })?
                }
                "anchors" => vector::<EntityId>(value, request, |_| Ok(0.into()))?,
                "entities" => vector::<(EntityId, EntityId)>(value, request, |_| Ok(0.into()))?,
                "relations" => {
                    vector::<(RelationId, RelationId)>(value, request, |_| Ok(0.into()))?
                }
                "anchor" | "birth" | "retirement" | "original_anchor" => 0.into(),
                _ => return Err(Refusal::NonCanonicalPayload),
            };
            add(&mut bytes, nested)
        })?;
        Ok(bytes)
    })
}

pub(super) fn authority(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    if !value.is_map() {
        return Ok(OwnerAllocation::default());
    }
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |variant, value| {
        if variant != "RetainedEntitySelection" {
            return Err(Refusal::NonCanonicalPayload);
        }
        add(
            &mut bytes,
            shared_root::<crate::history::data::CanonicalRetainedEntitySelection>()?,
        )?;
        value.fields(request, |field, value| match field {
            "source_branch" => add(&mut bytes, text(value, request)?),
            "source_head" => value.fields(request, |field, value| {
                if field == "parent_commit_ids" {
                    add(&mut bytes, vector::<u64>(value, request, |_| Ok(0.into()))?)?;
                }
                Ok(())
            }),
            "source_generation" | "source_truth_version" | "entity" | "kind" => Ok(()),
            _ => Err(Refusal::NonCanonicalPayload),
        })
    })?;
    Ok(bytes)
}
