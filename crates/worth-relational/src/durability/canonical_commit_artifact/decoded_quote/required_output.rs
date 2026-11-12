//! Producer membership may escape the envelope; quote its actual Arc owner.
use super::*;
use crate::history::data::{
    CanonicalOutputMember, CanonicalRequiredOutputMaterialization,
    CanonicalRequiredOutputMaterializationRecord, CanonicalRequiredOutputOrigin,
    CanonicalRequiredOutputRelation, CommitId,
};
use crate::identity::data::EntityId;

pub(super) fn origin(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_root::<CanonicalRequiredOutputOrigin>()?;
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "commit" => super::semantic::receipt(value, request)?,
                "basis" => super::branches::descriptor(value, request)?,
                "producer" | "birth_producer" => text(value, request)?,
                "meaning" => meaning(value, request)?,
                "outputs" => {
                    vector::<EntityId>(value, request, |_| Ok(OwnerAllocation::default()))?
                }
                "relations" => vector::<CanonicalRequiredOutputRelation>(value, request, |_| {
                    Ok(OwnerAllocation::default())
                })?,
                "members" => {
                    vector::<CanonicalOutputMember>(value, request, |value| member(value, request))?
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
pub(super) fn meaning(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if field == "family" {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
fn member(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = OwnerAllocation::default();
    value.fields(request, |field, value| {
        if matches!(field, "role" | "entity_name" | "birth_producer") {
            add(&mut bytes, text(value, request)?)?;
        }
        Ok(())
    })?;
    Ok(bytes)
}
pub(super) fn materialization(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_root::<CanonicalRequiredOutputMaterialization>()?;
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "producer_publications" => {
                    vector::<CommitId>(value, request, |_| Ok(OwnerAllocation::default()))?
                }
                "records" => {
                    vector::<CanonicalRequiredOutputMaterializationRecord>(value, request, |_| {
                        Ok(OwnerAllocation::default())
                    })?
                }
                _ => OwnerAllocation::default(),
            },
        )
    })?;
    Ok(bytes)
}
