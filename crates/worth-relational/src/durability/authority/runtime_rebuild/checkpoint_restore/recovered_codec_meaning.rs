//! Refuse changed current codec meaning before mirror reconstruction or indexes.
use crate::durability::data::{
    DurabilityError, DurableCheckpoint, RecoveryFailureClass, RelationalRecoveryStop,
};
use crate::schema::data::{DeclaredAspectContractBinding, RelationalSchemaRegistry};
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, MapKernelContext};
use worth_foundational::{facade::AspectShape, FoundationalBranchTarget};

/// Only native readmitted root authorities enter this comparison. Historical
/// roots retain their own meaning; current branch heads cannot adopt new codecs
/// through ordinary exact recovery. This fence grants no transition authority.
pub(super) fn check_current_codec_meaning(
    execution: &ExecutionRequest<'_, '_>,
    checkpoint: &DurableCheckpoint,
    roots: &super::branch_root_images::RestoredBranchRootImages,
    target: &RelationalSchemaRegistry,
) -> Result<(), RelationalRecoveryStop> {
    execution
        .in_scope(|lease| {
            ExecutionWorkCeiling::new(execution.work_ceiling()).run_with_checkpoint(
                lease,
                None,
                |context| {
                    for cell in &checkpoint.branch_cells {
                        charge(context, 1)?;
                        let FoundationalBranchTarget::Basis(selected) = cell.observation.target()
                        else {
                            continue;
                        };
                        let commit = crate::history::data::CommitId(selected.selected_commit_id());
                        charge(context, lookup_work(roots.schema_authorities.len()))?;
                        let source = roots.schema_authorities.get(&commit).ok_or_else(|| {
                            DurabilityError::new(
                                RecoveryFailureClass::CorruptCheckpoint,
                                "current branch has no readmitted schema carrier",
                            )
                        })?;
                        if source.authority_digest() != *selected.roots().schema_root() {
                            return Err(DurabilityError::new(
                                RecoveryFailureClass::CorruptCheckpoint,
                                "current branch schema carrier differs from its native target",
                            )
                            .into());
                        }
                        compare_registries(source.registry(), target, context)?;
                        // Symmetry refuses added as well as removed canonical carriers.
                        compare_registries(target, source.registry(), context)?;
                    }
                    Ok(())
                },
            )
        })
        .map_err(request)?
        .map_err(request)?
        .0
}

fn compare_registries(
    source: &RelationalSchemaRegistry,
    target: &RelationalSchemaRegistry,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    for (kind, previous) in &source.entity_kinds {
        charge(context, lookup_work(target.entity_kinds.len()))?;
        if let Some(next) = target.entity_kinds.get(kind) {
            compare_bindings(
                &previous.aspect_contract_declarations.aspects,
                &next.aspect_contract_declarations.aspects,
                context,
            )?;
        } else {
            for binding in &previous.aspect_contract_declarations.aspects {
                if has_codec(&binding.contract, context)? {
                    return Err(carrier_changed());
                }
            }
        }
    }
    for (kind, previous) in &source.relation_kinds {
        charge(context, lookup_work(target.relation_kinds.len()))?;
        if let Some(next) = target.relation_kinds.get(kind) {
            compare_bindings(
                &previous.aspect_contract_declarations.aspects,
                &next.aspect_contract_declarations.aspects,
                context,
            )?;
        } else {
            for binding in &previous.aspect_contract_declarations.aspects {
                if has_codec(&binding.contract, context)? {
                    return Err(carrier_changed());
                }
            }
        }
    }
    Ok(())
}

fn compare_bindings(
    previous: &[DeclaredAspectContractBinding],
    next: &[DeclaredAspectContractBinding],
    context: &mut MapKernelContext<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    for previous in previous {
        let old = &previous.contract;
        let mut matched = None;
        for candidate in next {
            charge_pair(
                context,
                1,
                old.key().as_str().len(),
                candidate.contract.key().as_str().len(),
            )?;
            charge_pair(
                context,
                1,
                previous.binding.owned_allocation_capacity_bytes(),
                candidate.binding.owned_allocation_capacity_bytes(),
            )?;
            if previous.binding == candidate.binding
                && old.key() == candidate.contract.key()
                && old.identity() == candidate.contract.identity()
            {
                matched = Some(&candidate.contract);
                break;
            }
        }
        let Some(new) = matched else {
            if has_codec(old, context)? {
                return Err(carrier_changed());
            }
            continue;
        };
        let (AspectShape::Struct(old_fields), AspectShape::Struct(new_fields)) =
            (old.shape(), new.shape())
        else {
            if has_codec(old, context)? {
                return Err(carrier_changed());
            }
            continue;
        };
        for old_field in old_fields.fields() {
            let mut matched = None;
            for new_field in new_fields.fields() {
                charge_pair(
                    context,
                    65,
                    old_field.key().as_str().len(),
                    new_field.key().as_str().len(),
                )?;
                if old_field.key() == new_field.key() {
                    matched = Some(new_field);
                    break;
                }
            }
            let Some(new_field) = matched else {
                if old_field.codec().is_some() {
                    return Err(carrier_changed());
                }
                continue;
            };
            if old_field.codec() == new_field.codec() {
                continue;
            }
            let reason = if old_field.codec().is_none() || new_field.codec().is_none() {
                "recovered field codec has missing meaning classification"
            } else if new.revision() <= old.revision() {
                "recovered field codec meaning changed without an advanced semantic aspect revision"
            } else {
                "recovered derived codec requires authenticated native transition admission"
            };
            return Err(DurabilityError::new(RecoveryFailureClass::SchemaMismatch, reason).into());
        }
    }
    Ok(())
}

fn has_codec(
    contract: &worth_foundational::facade::AspectContract,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<bool, RelationalRecoveryStop> {
    let AspectShape::Struct(fields) = contract.shape() else {
        return Ok(false);
    };
    charge(context, fields.fields().len())?;
    Ok(fields.fields().iter().any(|field| field.codec().is_some()))
}

fn carrier_changed() -> RelationalRecoveryStop {
    DurabilityError::new(
        RecoveryFailureClass::SchemaMismatch,
        "recovered canonical field carrier changed before exact native admission",
    )
    .into()
}

fn charge(
    context: &mut MapKernelContext<'_, '_>,
    work: usize,
) -> Result<(), RelationalRecoveryStop> {
    let work = u64::try_from(work).map_err(|_| overflow())?;
    context.checkpoint(work).map_err(request)
}

fn lookup_work(entries: usize) -> usize {
    // A B-tree has at most 11 key comparisons per node; binary depth is a
    // conservative bound on its actual minimum branching of at least two.
    1 + 11 * (usize::BITS - entries.leading_zeros()) as usize
}

fn charge_pair(
    context: &mut MapKernelContext<'_, '_>,
    base: usize,
    left: usize,
    right: usize,
) -> Result<(), RelationalRecoveryStop> {
    charge(
        context,
        base.checked_add(left)
            .and_then(|work| work.checked_add(right))
            .ok_or_else(overflow)?,
    )
}

fn overflow() -> RelationalRecoveryStop {
    RelationalRecoveryStop::Request(
        crate::execution::RelationalExecutionDenialCause::WorkCounterOverflow,
    )
}

fn request(
    cause: impl Into<crate::execution::RelationalExecutionDenialCause>,
) -> RelationalRecoveryStop {
    RelationalRecoveryStop::Request(cause.into())
}
