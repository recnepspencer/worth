//! Validate exact captured transport and decode its original schema carrier.
use super::*;
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use worth_relational_contracts::internal_port as contracts;

/// Pay for the roster visit, ordered lookup and variable receipt comparison
/// before touching the actual row. This also polls cancellation at each row.
pub fn receipt_lookup_work(
    receipt: &contracts::history::data::commit_receipt::RelationalCommitReceipt,
    source: &CaptureIdentity,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Refusal> {
    let levels = source.canonical.len().checked_ilog2().map_or(1, |n| n + 1);
    let work = receipt
        .branch_id
        .0
        .len()
        .checked_add(receipt.parents.len())
        .and_then(|n| n.checked_add(levels as usize))
        .and_then(|n| n.checked_add(1))
        .ok_or(Refusal::CapacityOverflow)?;
    contracts::durability::data::recovery_stop::account_work(
        request,
        u64::try_from(work).map_err(|_| Refusal::CapacityOverflow)?,
    )
    .map_err(contracts::durability::checkpoint_admission::refusal::native_stop)
}

pub fn decode_origin(
    origin: &CapturedNativeInterimOrigin,
    durable_receiver: bool,
    registry: &contracts::schema::data::RelationalSchemaRegistry,
    request: &ExecutionRequest<'_, '_>,
) -> Result<crate::durability::data::DurableCheckpoint, RelationalRecoveryStop> {
    if durable_receiver {
        return Err(RelationalRecoveryStop::AdmissionImage(
            Refusal::BindingMismatch(Mismatch::Scope),
        ));
    }
    origin
        .validate_transport(origin.0.wrapper.bytes(), request)
        .map_err(RelationalRecoveryStop::AdmissionImage)?;
    let native = &origin.0.wrapper.bytes()[origin.0.native.clone()];
    let image = &origin.0.wrapper.bytes()[origin.0.image_region.clone()];
    if image != origin.0.image.as_bytes() {
        return Err(RelationalRecoveryStop::AdmissionImage(
            Refusal::MissingCaptureOrigin,
        ));
    }
    let mut checkpoint =
        crate::durability::log::native_checkpoint_codec::decode_interim_with_image(
            request,
            native,
            origin.0.image.clone(),
        )?
        .checkpoint;
    let copies = usize::from(checkpoint.directory.is_some())
        + usize::from(checkpoint.canonical_directory.is_some());
    let mut schemas = crate::durability::source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog::prepare(request, copies)?;
    for image in &checkpoint.branch_root_schema_images {
        crate::durability::source_initial_inputs::schema_catalog::ReadmittedSourceSchemaCatalog::insert_image(
            &mut schemas, image, registry, request)?;
    }
    for directory in checkpoint
        .directory
        .iter_mut()
        .chain(checkpoint.canonical_directory.iter_mut())
    {
        *directory = crate::durability::data::NativeCheckpointDirectoryEdge::new(
            directory.with_source_schemas(schemas.clone(), request)?,
        );
    }
    Ok(checkpoint)
}

/// The selected directory must carry this actual capture-issued image. Its
/// root-directory projections were joined to this manifest by decode_origin.
pub(in crate::internal_port::durability::authority) fn decoded_origin_directory(
    origin: &CapturedNativeInterimOrigin,
    checkpoint: &crate::durability::data::DurableCheckpoint,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Arc<crate::durability::data::RelationalCheckpointDirectory>, RelationalRecoveryStop> {
    request
        .consult()
        .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
    let directory = checkpoint
        .directory
        .as_ref()
        .and_then(|edge| edge.snapshot())
        .ok_or(RelationalRecoveryStop::AdmissionImage(
            Refusal::MissingArtifact,
        ))?;
    if !directory
        .admission_image
        .as_ref()
        .is_some_and(|image| image.same_owner(&origin.0.image))
    {
        return Err(RelationalRecoveryStop::AdmissionImage(
            Refusal::MissingCaptureOrigin,
        ));
    }
    Ok(directory)
}
