//! Compare carried producer custody with the original complete canonical meaning.
use super::{
    origin_request, CanonicalRequiredOutputOrigin, CanonicalRequiredOutputRelation,
    TransactionCommitError,
};
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
        context
            .checkpoint(u64::try_from(work).map_err(|_| {
                origin_request(
                    crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
                )
            })?)
            .map_err(origin_request)?;
    }
    if carried != canonical {
        return Err(crate::mvcc::origin_denial(
            "retained publication custody differs from exact canonical membership",
        ));
    }
    Ok(())
}
