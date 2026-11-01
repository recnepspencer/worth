use super::*;
use crate::authority::mutation::MutationApplyDenial;
use crate::runtime::MutationSymbolMeanings;
use crate::schema::data::LoweredAspectContractBinding;
use crate::transactions::data::TransactionCommitError;

pub(in crate::authority::mutation) fn write_original_field_versions_for_binding(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    staged: &mut WorkingState,
    target: RecordRef,
    binding: &LoweredAspectContractBinding,
    evidence: &BorrowedAspectDeltaEvidence<'_>,
    structural: crate::publication::patch::data::RecordStructuralChange,
    symbols: &mut dyn MutationSymbolMeanings,
    version: VersionId,
) -> Result<(), MutationApplyDenial> {
    let mut aspect = None;
    visit_borrowed_field_changes(
        &binding.target,
        binding.contract.shape(),
        evidence,
        structural,
        |field, _| {
            if aspect.is_none() {
                aspect = Some(symbols.reserve_revision_reference(binding.aspect_key().as_str())?);
            }
            symbols.reserve_revision_reference(field.as_str())?;
            Ok::<_, crate::runtime::SymbolReservationDenial>(())
        },
    )?;
    let Some(aspect) = aspect else {
        return Ok(());
    };
    match target {
        RecordRef::Entity(entity) => {
            let partition = staged
                .get_partition_mut_for_request(partition_of::<EntityRecordKind>(&entity), request)
                .map_err(execution)?;
            let Some(revisions) = partition
                .entity_arena
                .field_revisions_at_mut_using(slot_of::<EntityRecordKind>(&entity), Some(request))
                .map_err(execution)?
            else {
                return Ok(());
            };
            visit_borrowed_field_changes(
                &binding.target,
                binding.contract.shape(),
                evidence,
                structural,
                |field, presence| {
                    let symbol=symbols.symbol(field.as_str()).expect(/* Proof: The first traversal admitted these exact borrowed fields in the same immutable evidence and binding before exposing this writer. */ "field was admitted above");
                    revisions
                        .insert_for_request(
                            (aspect, symbol),
                            RelationalFieldRevision::new(version, presence),
                            request,
                        )
                        .map_err(execution)?;
                    Ok::<_, MutationApplyDenial>(())
                },
            )
        }
        RecordRef::Relation(relation) => {
            let partition = staged
                .get_partition_mut_for_request(
                    partition_of::<RelationRecordKind>(&relation),
                    request,
                )
                .map_err(execution)?;
            let Some(revisions) = partition
                .relation_arena
                .field_revisions_at_mut_using(
                    slot_of::<RelationRecordKind>(&relation),
                    Some(request),
                )
                .map_err(execution)?
            else {
                return Ok(());
            };
            visit_borrowed_field_changes(
                &binding.target,
                binding.contract.shape(),
                evidence,
                structural,
                |field, presence| {
                    let symbol=symbols.symbol(field.as_str()).expect(/* Proof: The first traversal admitted these exact borrowed fields in the same immutable evidence and binding before exposing this writer. */ "field was admitted above");
                    revisions
                        .insert_for_request(
                            (aspect, symbol),
                            RelationalFieldRevision::new(version, presence),
                            request,
                        )
                        .map_err(execution)?;
                    Ok::<_, MutationApplyDenial>(())
                },
            )
        }
    }
}
fn execution(cause: crate::execution::RelationalExecutionDenialCause) -> MutationApplyDenial {
    TransactionCommitError::execution(cause.into()).into()
}
