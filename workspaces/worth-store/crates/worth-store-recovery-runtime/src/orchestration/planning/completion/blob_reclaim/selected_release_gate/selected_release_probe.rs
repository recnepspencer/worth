//! Inspect the actual selected routes and exact V2 descriptor bytes.
use super::super::record::selected_record;
use super::{PlanningContext, ResolvedPlanningState};
use worth_store_physical_format::{BlobRecordKind, BlobRecordV1, SelectedRecordContentClass};

pub(super) fn selected_has_release(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningState,
) -> Result<(PlanningContext, bool), crate::entry::PhysicalRecoveryOutcome> {
    for index in 0..context.selection.page_facts().placements().len() {
        let route = context.selection.page_facts().placements()[index];
        match route.content_class() {
            SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV3) => {
                return Ok((context, true))
            }
            SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV2) => {
                let generation = context.selection.root().selected().manifest().generation();
                let (next, bytes) = selected_record(
                    context,
                    basis,
                    generation,
                    route.record(),
                    worth_store_physical_format::BLOB_CONTROL_FRAME_MAX_BYTES as u64,
                )?;
                context = next;
                match worth_store_physical_format::decode_blob_record(&bytes) {
                    Ok(BlobRecordV1::ReclaimDescriptorV2(value))
                        if value.source_kind()
                            == worth_store_physical_format::BlobReclaimSourceKind::FailedIngest => {
                    }
                    Ok(BlobRecordV1::ReclaimDescriptorV2(_)) => return Ok((context, true)),
                    Ok(
                        BlobRecordV1::SessionDeclared(_)
                        | BlobRecordV1::Chunk(_)
                        | BlobRecordV1::TreeNode(_)
                        | BlobRecordV1::GenerationPublished(_)
                        | BlobRecordV1::SessionFrontier(_)
                        | BlobRecordV1::SessionAbandoned(_)
                        | BlobRecordV1::DropSetManifest(_)
                        | BlobRecordV1::ReclaimDescriptor(_)
                        | BlobRecordV1::DropSetManifestV2(_)
                        | BlobRecordV1::OriginalDropReserved(_)
                        | BlobRecordV1::ChunkReuseClaim(_)
                        | BlobRecordV1::DedupeQuarantine(_)
                        | BlobRecordV1::DropSetManifestV3(_)
                        | BlobRecordV1::ChunkReuseClaimV2(_)
                        | BlobRecordV1::ReclaimDescriptorV3(_),
                    ) => return Err(context.redo_block(basis.planning_counters(), None)),
                    Err(_cause) => return Err(context.redo_block(basis.planning_counters(), None)),
                }
            }
            SelectedRecordContentClass::UnknownLegacy
            | SelectedRecordContentClass::Opaque
            | SelectedRecordContentClass::DerivedDirectory
            | SelectedRecordContentClass::BTreeNode { .. }
            | SelectedRecordContentClass::Blob(
                BlobRecordKind::SessionDeclared
                | BlobRecordKind::Chunk
                | BlobRecordKind::TreeNode
                | BlobRecordKind::GenerationPublished
                | BlobRecordKind::SessionFrontier
                | BlobRecordKind::SessionAbandoned
                | BlobRecordKind::DropSetManifest
                | BlobRecordKind::ReclaimDescriptor
                | BlobRecordKind::DropSetManifestV2
                | BlobRecordKind::OriginalDropReserved
                | BlobRecordKind::ChunkReuseClaim
                | BlobRecordKind::DedupeQuarantine
                | BlobRecordKind::DropSetManifestV3
                | BlobRecordKind::ChunkReuseClaimV2,
            ) => {}
        }
    }
    Ok((context, false))
}
