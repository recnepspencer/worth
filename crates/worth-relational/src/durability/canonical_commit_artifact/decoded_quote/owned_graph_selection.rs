//! Graph-selection DATA is funded before its concrete entity and edge vectors decode.
use super::*;
use crate::history::data::CanonicalOwnedGraphSelection;
use crate::identity::data::EntityId;
use crate::validation::data::StructuralRelationRecord;

pub(super) fn quote(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<OwnerAllocation, Refusal> {
    let mut bytes = shared_root::<CanonicalOwnedGraphSelection>()?;
    value.fields(request, |field, value| {
        add(
            &mut bytes,
            match field {
                "selected_basis" => super::branches::descriptor(value, request)?,
                "current_entities" | "transfer_participants" => {
                    vector::<EntityId>(value, request, |_| Ok(0.into()))?
                }
                "selected_entities" => vector_with_storage(
                    value,
                    request,
                    CanonicalOwnedGraphSelection::selected_entity_decode_storage,
                    |_| Ok(0.into()),
                )?,
                "current_relations" | "selected_relations" => {
                    vector::<StructuralRelationRecord>(value, request, |_| Ok(0.into()))?
                }
                "model" => OwnerAllocation::default(),
                _ => return Err(Refusal::NonCanonicalPayload),
            },
        )
    })?;
    Ok(bytes)
}
