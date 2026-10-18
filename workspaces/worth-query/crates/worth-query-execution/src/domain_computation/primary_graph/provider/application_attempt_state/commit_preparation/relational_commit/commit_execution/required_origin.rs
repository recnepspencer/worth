//! Attribute only the actual created roles of an admitted running producer.
use super::*;
use crate::domain_computation::primary_graph::{
    output_lineage::invalidation::InvalidationEditAdmission, PreparedPrerequisiteClaims,
};
use worth_relational::facade::mvcc::{CompanionPreflightStop, ValidatedRelationalProposal};

pub(super) fn bind(
    proposal: ValidatedRelationalProposal,
    provider: &WorthQueryPrimaryGraphProvider,
    attempt: &WorthQueryPrimaryGraphApplicationAttempt,
    required: Option<&PreparedPrerequisiteClaims>,
    admission: &mut InvalidationEditAdmission,
) -> Result<
    ValidatedRelationalProposal,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    let Some(required) = required else {
        return Ok(proposal);
    };
    let Some(publisher) = &provider.graph.application_publication else {
        // Low-level graphs without an installed application publisher never
        // acquire reconstructibility from an output role alone.
        return Ok(proposal);
    };
    let Some(dependency) = attempt.idempotency().producer_dependency_identity() else {
        return Ok(proposal);
    };
    let overflow =
        || native_output_witness_stop(CompanionPreflightStop::PreparationMemoryCounterOverflow);
    let targets = attempt.native_created_output_targets();
    let examined = targets.size_hint().1.ok_or_else(overflow)?;
    admission
        .charge_external_work(
            u64::try_from(examined)
                .map_err(|_| overflow())?
                .checked_mul(3)
                .ok_or_else(overflow)?,
        )
        .map_err(native_output_witness_stop)?;
    let count = targets.clone().count();
    if count == 0 {
        return Ok(proposal);
    }
    let (producer, _) = required.native_prior_checkpoint_input();
    let mut bytes = count
        .checked_mul(std::mem::size_of::<
            worth_relational::facade::transactions::CreatedEntityRef,
        >())
        .and_then(|bytes| bytes.checked_add(producer.len()))
        .and_then(|bytes| bytes.checked_add(2 * std::mem::size_of::<usize>()))
        .ok_or_else(overflow)?;
    for target in targets.clone() {
        bytes = bytes
            .checked_add(target.client_key.as_raw_str().map_or(0, str::len))
            .ok_or_else(overflow)?;
    }
    let bytes = u64::try_from(bytes).map_err(|_| overflow())?;
    admission
        .admit_read_scratch(bytes)
        .map_err(native_output_witness_stop)?;
    admission
        .charge_external_work(bytes.checked_add(count as u64).ok_or_else(overflow)?)
        .map_err(native_output_witness_stop)?;
    // The filtered iterator has no exact lower size hint. An ordinary collect
    // can grow beyond the backing admitted above, even for one output.
    let mut selected = Vec::new();
    let backing_refused = || {
        crate::domain_computation::WorthQueryProviderSessionCommitStop::PreEffectDenied(failure(
            "native output target backing could not meet its admitted exact capacity",
        ))
    };
    selected
        .try_reserve_exact(count)
        .map_err(|_| backing_refused())?;
    if selected.capacity() != count {
        return Err(backing_refused());
    }
    selected.extend(targets.cloned());
    let proof = publisher.owner.admit_required_output(producer, dependency);
    proposal
        .bind_required_output_publication(&publisher.native, proof, selected)
        .map_err(transaction_commit_stop)
}
