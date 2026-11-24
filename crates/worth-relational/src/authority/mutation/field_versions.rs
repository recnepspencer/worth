use super::canonical_deltas::field_changes::changed_fields;
pub(crate) use super::canonical_deltas::field_changes::visit_changed_fields;
use crate::identity::data::VersionId;
use crate::storage::data::RelationalFieldRevision;
use crate::storage::overlay::WorkingState;
use crate::storage::substrate::{partition_of, slot_of, EntityRecordKind, RelationRecordKind};
use crate::transactions::data::primitives::RecordRef;

/// Stamp owner-native field transitions on the same staged root that receives
/// the canonical aspect delta. Unknown restored columns stay unknown until a
/// checkpoint image version can carry their prior revisions.
pub(super) fn write_field_versions_for_delta(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    staged: &mut WorkingState,
    admitted: &super::delta_symbols::AdmittedDeltaSymbols<'_, '_>,
    version: VersionId,
) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
    let delta = admitted.delta();
    for binding in &delta.evaluated_bindings {
        let fields = changed_fields(binding, delta.structural_change);
        if fields.is_empty() {
            continue;
        }
        let aspect = admitted.aspect(&binding.aspect_key);
        match delta.target {
            RecordRef::Entity(entity) => {
                let partition = staged.get_partition_mut_for_request(
                    partition_of::<EntityRecordKind>(&entity),
                    request,
                )?;
                let Some(revisions) = partition.entity_arena.field_revisions_at_mut_using(
                    slot_of::<EntityRecordKind>(&entity),
                    Some(request),
                )?
                else {
                    continue;
                };
                for (field, presence) in fields {
                    revisions.insert_for_request(
                        (aspect, admitted.field(&field)),
                        RelationalFieldRevision::new(version, presence),
                        request,
                    )?;
                }
            }
            RecordRef::Relation(relation) => {
                let partition = staged.get_partition_mut_for_request(
                    partition_of::<RelationRecordKind>(&relation),
                    request,
                )?;
                let Some(revisions) = partition.relation_arena.field_revisions_at_mut_using(
                    slot_of::<RelationRecordKind>(&relation),
                    Some(request),
                )?
                else {
                    continue;
                };
                for (field, presence) in fields {
                    revisions.insert_for_request(
                        (aspect, admitted.field(&field)),
                        RelationalFieldRevision::new(version, presence),
                        request,
                    )?;
                }
            }
        }
    }
    Ok(())
}
