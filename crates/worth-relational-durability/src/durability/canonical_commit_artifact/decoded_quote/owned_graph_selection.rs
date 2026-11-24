//! Graph-selection DATA is funded before its concrete entity and edge vectors decode.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::owned_graph_selection::{CanonicalOwnedGraphSelection, CanonicalOwnedGraphSelectionOwnerPort as _};
use super::*;
use worth_relational_contracts::internal_port::identity::data::record_ids::EntityId;
use worth_relational_contracts::internal_port::validation::data::structural_read::StructuralRelationRecord;

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
