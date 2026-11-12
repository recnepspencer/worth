//! Admission joins the capture origin to this actual recovered Native owner.
use super::*;
use crate::branch::{ObservedRelationalMemoryComponent, RecordedRelationalComponentTarget};
use crate::durability::data::RelationalRecoveryStop;
use crate::runtime::RelationalRuntime;

#[derive(Clone, Debug)]
pub struct AdmittedNativeInterimReceiver {
    pub(super) source: Arc<CaptureIdentity>,
    pub(super) owner: Weak<crate::runtime::RelationalRuntimeState>,
    pub(super) image: RelationalCheckpointAdmissionArtifact,
}
impl AdmittedNativeInterimReceiver {
    pub(crate) fn readmit_component(
        &self,
        runtime: &RelationalRuntime,
        captured: &CapturedNativeInterimComponent,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<ObservedRelationalMemoryComponent, Refusal> {
        request_scope::run(request, || {
            request
                .consult()
                .map_err(|cause| Refusal::Request(cause.into()))?;
            if !Weak::ptr_eq(&self.owner, &runtime.state_binding())
                || !captured.belongs_to(&self.source)
            {
                return Err(Refusal::BindingMismatch(Mismatch::Runtime));
            }
            let directory = runtime
                .durability
                .checkpoint_selection()
                .require_interim_image(&self.image)?;
            let admitted = match captured.description().target() {
                RecordedRelationalComponentTarget::Empty => {
                    let schema = captured.description().descriptor().schema_commitment();
                    if !directory
                        .manifest
                        .schemas
                        .iter()
                        .any(|image| image.schema_root() == schema)
                    {
                        return Err(Refusal::InvalidProjectedMeaning);
                    }
                    None
                }
                RecordedRelationalComponentTarget::Canonical(target) => {
                    receipt_lookup_work(target.receipt(), &self.source, request)?;
                    let index = self
                        .source
                        .canonical
                        .get(&target.receipt().commit_id)
                        .ok_or(Refusal::InvalidProjectedMeaning)?;
                    Some(require_row(runtime, index, &self.source, request)?)
                }
            };
            ObservedRelationalMemoryComponent::from_interim(
                request,
                self.owner.clone(),
                captured.description().clone(),
                admitted,
            )
        })
    }
}
fn require_row(
    runtime: &RelationalRuntime,
    index: &crate::history::RelationalCommitIndexEntry,
    source: &CaptureIdentity,
    request: &ExecutionRequest<'_, '_>,
) -> Result<crate::durability::checkpoint_admission::CheckpointAdmittedCommit, Refusal> {
    receipt_lookup_work(&index.commit, source, request)?;
    let artifact = runtime
        .history
        .commit_artifact(index.commit.commit_id)
        .ok_or(Refusal::OwnerUnavailable)?;
    let row = runtime
        .durability
        .checkpoint_selection()
        .admitted_commit(index)?
        .ok_or(Refusal::MissingCaptureOrigin)?;
    let matches = match row.scope() {
        CheckpointAdmissionScope::Interim {
            transport_origin, ..
        } => transport_origin == source.coordinate.bytes(),
        CheckpointAdmissionScope::Home { .. } => false,
    };
    if !matches || row.index() != index || !row.links_artifact(&artifact) {
        return Err(Refusal::BindingMismatch(Mismatch::NativeOrigin));
    }
    Ok(row)
}
impl crate::durability::authority::DurabilityAuthority<'_> {
    pub fn admit_interim_receiver(
        &self,
        origin: &CapturedNativeInterimOrigin,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<AdmittedNativeInterimReceiver, Refusal> {
        request_scope::run(request, || {
            request
                .consult()
                .map_err(|cause| Refusal::Request(cause.into()))?;
            let _operation = self
                .runtime
                .owner_binding()
                .admit()
                .ok_or(Refusal::OwnerUnavailable)?;
            if self.runtime.durability.medium().is_some() {
                return Err(Refusal::BindingMismatch(Mismatch::Scope));
            }
            self.runtime
                .durability
                .checkpoint_selection()
                .require_interim_image(&origin.0.image)?;
            for index in origin.0.source.canonical.values() {
                require_row(self.runtime, index, &origin.0.source, request)?;
            }
            Ok(AdmittedNativeInterimReceiver {
                source: origin.0.source.clone(),
                owner: self.runtime.state_binding(),
                image: origin.0.image.clone(),
            })
        })
    }
}

/// Pay for the roster visit, ordered lookup and variable receipt comparison
/// before touching the actual row. This also polls cancellation at each row.
pub(super) fn receipt_lookup_work(
    receipt: &crate::history::data::RelationalCommitReceipt,
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
    crate::durability::artifact_payload_emission::request::account_work(
        request,
        u64::try_from(work).map_err(|_| Refusal::CapacityOverflow)?,
    )
    .map_err(crate::durability::checkpoint_admission::native_stop)
}

pub(in crate::durability::authority) fn decode_origin(
    origin: &CapturedNativeInterimOrigin,
    runtime: &RelationalRuntime,
    request: &ExecutionRequest<'_, '_>,
) -> Result<crate::durability::data::DurableCheckpoint, RelationalRecoveryStop> {
    if runtime.durability.medium().is_some() {
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
            &mut schemas, image, &runtime.config().schema.registry, request)?;
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

impl crate::durability::authority::DurabilityRecoveryAuthority<'_> {
    pub fn restore_interim_checkpoint_for_codec_transition_with_authority(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        origin: &CapturedNativeInterimOrigin,
    ) -> Result<
        (
            crate::runtime::RecoveryOutcome,
            crate::durability::authority::RecoveredRelationalRuntimeAuthority,
        ),
        RelationalRecoveryStop,
    > {
        request_scope::run_recovery(request, || {
            crate::execution::check_request(request).map_err(RelationalRecoveryStop::Request)?;
            self.runtime
                .admit_codec_recovery_destination()
                .map_err(|detail| {
                    crate::durability::data::DurabilityError::new(
                        crate::durability::data::RecoveryFailureClass::ReplayFailure,
                        detail,
                    )
                })?;
            let checkpoint = decode_origin(origin, self.runtime, request)?;
            let restored =
                self.restore_decoded_interim_for_codec_transition(request, checkpoint, origin)?;
            // Codec recovery may replace the candidate runtime. Bind originals
            // to that actual receiver, never to the discarded target candidate.
            install_originals(self.runtime, origin, request)?;
            Ok(restored)
        })
    }

    pub fn restore_interim_checkpoint_with_authority(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        origin: &CapturedNativeInterimOrigin,
    ) -> Result<
        (
            crate::runtime::RecoveryOutcome,
            crate::durability::authority::RecoveredRelationalRuntimeAuthority,
        ),
        RelationalRecoveryStop,
    > {
        request_scope::run_recovery(request, || {
            let checkpoint = decode_origin(origin, self.runtime, request)?;
            let restored = self.restore_decoded_native_checkpoint_with_authority(
                request,
                checkpoint,
                origin.0.native.len(),
            )?;
            install_originals(self.runtime, origin, request)?;
            Ok(restored)
        })
    }
}

fn install_originals(
    runtime: &RelationalRuntime,
    origin: &CapturedNativeInterimOrigin,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    // Unconditional, including an empty capture: original DATA cannot
    // substitute for the exact image actually installed on this owner.
    runtime
        .durability
        .checkpoint_selection()
        .require_interim_image(&origin.0.image)
        .map_err(RelationalRecoveryStop::AdmissionImage)?;
    for original in origin.0.originals.iter() {
        receipt_lookup_work(&original.commit, &origin.0.source, request)
            .map_err(RelationalRecoveryStop::AdmissionImage)?;
        let index = origin
            .0
            .source
            .canonical
            .get(&original.commit.commit_id)
            .ok_or(RelationalRecoveryStop::AdmissionImage(
                Refusal::InvalidProjectedMeaning,
            ))?;
        let row = require_row(runtime, index, &origin.0.source, request)
            .map_err(RelationalRecoveryStop::AdmissionImage)?;
        let artifact = runtime
            .history
            .commit_artifact(index.commit.commit_id)
            .ok_or(RelationalRecoveryStop::AdmissionImage(
                Refusal::OwnerUnavailable,
            ))?;
        artifact
            .envelope_residency()
            .bind_interim_original(&row, original, request)
            .map_err(RelationalRecoveryStop::AdmissionImage)?;
    }
    Ok(())
}

/// The selected directory must carry this actual capture-issued image. Its
/// root-directory projections were joined to this manifest by decode_origin.
pub(in crate::durability::authority) fn decoded_origin_directory(
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
