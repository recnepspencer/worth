//! Current branch heads must retain the meaning admitted by exact recovery.
use super::recovered_codec_meaning::{charge, lookup_work, request};
use super::{recovered_codec_meaning, recovered_relation_laws};
use crate::durability::data::{DurableCheckpoint, RelationalRecoveryStop};
use crate::schema::data::RelationalSchemaRegistry;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling};
use worth_foundational::FoundationalBranchTarget;

/// Only carriers bound to a current cell's exact schema root enter this
/// comparison; binding corruption is refused by exact readmission. Historical
/// roots retain their own meaning; current branch heads cannot adopt new codecs
/// or relation laws through ordinary exact recovery. This fence grants no transition authority.
pub(super) fn check_current_schema_meaning(
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
                        // A cell without a carrier bound to its exact target is
                        // not a readmitted root authority. Exact branch-cell
                        // readmission owns that refusal (missing commit
                        // artifact, root image or schema root) and reaches it
                        // before any recovered state is installed.
                        let Some(source) = roots.schema_authorities.get(&commit).filter(|source| {
                            source.authority_digest() == *selected.roots().schema_root()
                        }) else {
                            continue;
                        };
                        let source_content = source.registry().content_formats.snapshot();
                        let target_content = target.content_formats.snapshot();
                        let bytes = source_content
                            .owned_bytes()
                            .checked_add(target_content.owned_bytes())
                            .and_then(|bytes| usize::try_from(bytes).ok())
                            .ok_or_else(|| {
                                crate::durability::data::DurabilityError::new(
                                    crate::durability::data::RecoveryFailureClass::SchemaMismatch,
                                    "content meaning size overflow",
                                )
                            })?;
                        charge(context, bytes)?;
                        if source_content != target_content {
                            return Err(crate::durability::data::DurabilityError::new(
                                crate::durability::data::RecoveryFailureClass::SchemaMismatch,
                                "current content format, version, ceiling or field binding changed",
                            )
                            .into());
                        }
                        recovered_codec_meaning::compare_registries(
                            source.registry(),
                            target,
                            context,
                        )?;
                        recovered_relation_laws::compare_registries(
                            source.registry(),
                            target,
                            context,
                        )?;
                        // Symmetry refuses added as well as removed canonical carriers.
                        recovered_codec_meaning::compare_registries(
                            target,
                            source.registry(),
                            context,
                        )?;
                        recovered_relation_laws::compare_registries(
                            target,
                            source.registry(),
                            context,
                        )?;
                    }
                    Ok(())
                },
            )
        })
        .map_err(request)?
        .map_err(request)?
        .0
}
