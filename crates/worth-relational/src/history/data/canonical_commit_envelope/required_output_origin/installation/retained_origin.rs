//! Compare carried producer custody with the original complete canonical meaning.
use super::{
    origin_request, CanonicalRequiredOutputOrigin, CanonicalRequiredOutputRelation,
    TransactionCommitError,
};
use crate::execution::request_stage::checkpoint;
use worth_execution::MapKernelContext;

pub(super) fn compare_retained_origin(
    carried: &CanonicalRequiredOutputOrigin,
    canonical: &CanonicalRequiredOutputOrigin,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), TransactionCommitError> {
    for origin in [carried, canonical] {
        let mut work = std::mem::size_of_val(origin);
        for bytes in [
            origin.producer.len(),
            origin.birth_producer.len(),
            origin.meaning.family.len(),
            origin.basis.branch_id().0.len(),
            origin
                .relations
                .len()
                .checked_mul(std::mem::size_of::<CanonicalRequiredOutputRelation>())
                .ok_or_else(|| {
                    origin_request(
                        crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                    )
                })?,
        ] {
            work = work.checked_add(bytes).ok_or_else(|| {
                origin_request(
                    crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                )
            })?;
        }
        for member in &origin.members {
            for bytes in [
                std::mem::size_of_val(member),
                member.role.len(),
                member.entity_name.len(),
                member.birth_producer.len(),
            ] {
                work = work.checked_add(bytes).ok_or_else(|| {
                    origin_request(
                        crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                    )
                })?;
            }
        }
        checkpoint(
            context,
            u64::try_from(work).map_err(|_| {
                origin_request(
                    crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                )
            })?,
        )
        .map_err(origin_request)?;
    }
    if carried != canonical {
        if canonical.source_scope_owned {
            use crate::{
                publication::data::{AuthoredRetirementRefusal, PublicationError},
                transactions::data::RecordRef,
            };
            let mut subject = RecordRef::Entity(canonical.meaning.source_scope);
            for (index, member) in canonical.members.iter().enumerate() {
                let other = carried.members.get(index);
                checkpoint(
                    context,
                    (std::mem::size_of_val(member)
                        + member.role.len()
                        + member.entity_name.len()
                        + member.birth_producer.len()
                        + other.map_or(0, |other| {
                            other.role.len() + other.entity_name.len() + other.birth_producer.len()
                        })) as u64,
                )
                .map_err(origin_request)?;
                if other != Some(member) {
                    subject = RecordRef::Entity(member.id);
                    break;
                }
            }
            checkpoint(
                context,
                canonical
                    .relations
                    .len()
                    .saturating_mul(std::mem::size_of::<CanonicalRequiredOutputRelation>())
                    as u64,
            )
            .map_err(origin_request)?;
            if subject == RecordRef::Entity(canonical.meaning.source_scope)
                && let Some((_, relation)) = canonical
                    .relations
                    .iter()
                    .enumerate()
                    .find(|(index, relation)| carried.relations.get(*index) != Some(*relation))
            {
                subject = RecordRef::Relation(relation.id);
            }
            return Err(TransactionCommitError::publication(
                PublicationError::authored_retirement(
                    AuthoredRetirementRefusal::CorruptPublication,
                    subject,
                ),
            ));
        }
        return Err(crate::mvcc::origin_denial(
            "retained publication custody differs from exact canonical membership",
        ));
    }
    Ok(())
}
