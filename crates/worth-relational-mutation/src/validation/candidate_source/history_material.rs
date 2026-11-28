//! Spend the original candidate budget while retaining the native historical material.
use worth_relational_branch::internal_port::validation::candidate_source::image::ManagedIndexBranchImageOwnerPort as _;
use super::image::{HistorySourceDirectory, ManagedIndexBranchImage};
use crate::contracts::identity::data::record_ids::EntityId;
use crate::contracts::validation::candidate_source::availability::ManagedIndexSourceAvailability as Availability;
use crate::contracts::validation::candidate_source::program::{
    ManagedIndexSourceProgram as Program, ManagedIndexSourceProgramFieldsPort as _,
};

/// Original-paid retention of exact immutable historical material only.
pub(crate) fn retained_history_receipt(
    image: &ManagedIndexBranchImage,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    program: &Program,
    scope: EntityId,
    budget: &crate::validation::custom_rule::CustomPreparationBudget,
) -> Result<
    Option<
        crate::branch_port::validation::candidate_source::opaque_custody::FundedSourceMaterialReceipt,
    >,
    super::ManagedIndexSourcePreparationDenial,
>{
    let roots = image.history_directory();
    let identity = &program.owner_field_0().path.parts().identity;
    let overflow = || {
        crate::contracts::derived_relation::read_denial::DerivedRelationReadDenial::Request(
            crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::WorkCounterOverflow,
        )
    };
    // SourceKey compares declared identity strings before fixed-width scope.
    // A comparison cannot inspect more than this supplied key's byte width.
    let width = identity
        .schema
        .0
        .len()
        .checked_add(identity.name.len())
        .and_then(|n| n.checked_add(HistorySourceDirectory::key_storage_bytes()))
        .ok_or_else(overflow)?;
    let height = 2 * (usize::BITS - roots.len().leading_zeros()) as usize + 2;
    let work = height.checked_mul(width.max(1)).ok_or_else(overflow)?;
    if !budget.try_visit(work) {
        return Err(
            crate::contracts::derived_relation::read_denial::DerivedRelationReadDenial::Definition(
                crate::contracts::derived_relation::definition::DerivedRelationDenial::WorkBudgetExceeded,
            )
            .into(),
        );
    }
    let Some(entry) = roots.selected(program, scope)? else {
        return Ok(None);
    };
    if entry.physical_availability() != Availability::Available {
        return Ok(None);
    }
    let Some(payload) = entry.payload() else {
        return Ok(None);
    };
    // Borrow under the actual directory guard: no temporary strong owner
    // can become last if another thread evicts immediately after unlock.
    let receipt = crate::branch_port::validation::candidate_source::opaque_custody::FundedSourceMaterialReceipt::retain(execution, payload)
        .map_err(crate::contracts::derived_relation::read_denial::DerivedRelationReadDenial::Request)?;
    Ok(Some(receipt))
}
