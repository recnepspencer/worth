use super::*;
pub(in crate::authority::mutation) fn write_original_aspect_version(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    staged: &mut WorkingState,
    target: RecordRef,
    aspect: crate::symbols::data::Symbol,
    version: VersionId,
) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
    match target {
        RecordRef::Entity(entity) => {
            let partition = staged.get_partition_mut_for_request(
                partition_of::<EntityRecordKind>(&entity),
                request,
            )?;
            let versions=partition.entity_arena.aspect_versions_at_mut_using(slot_of::<EntityRecordKind>(&entity),Some(request))?
                .expect(/* Proof: Original sealed mutation application retains the materialized target and initializes its version columns before the shared revision writer. */ "materialized entity has aspect versions");
            versions.insert_for_request(aspect, version.0, request)?;
        }
        RecordRef::Relation(relation) => {
            let partition = staged.get_partition_mut_for_request(
                partition_of::<RelationRecordKind>(&relation),
                request,
            )?;
            let versions=partition.relation_arena.aspect_versions_at_mut_using(slot_of::<RelationRecordKind>(&relation),Some(request))?
                .expect(/* Proof: Original sealed mutation application retains the materialized target and initializes its version columns before the shared revision writer. */ "materialized relation has aspect versions");
            versions.insert_for_request(aspect, version.0, request)?;
        }
    }
    Ok(())
}
