//! Admission joins the capture origin to this actual recovered Native owner.
use super::origin_readmission::{decode_origin, receipt_lookup_work};
use super::*;
use crate::branch::recorded_basis::{
    ObservedRelationalMemoryComponent, RecordedRelationalComponentTarget,
};
use crate::durability::authority::recovered_runtime::DurabilityRecoveryAuthority;
use crate::durability::authority::recovered_runtime::RecoveredRelationalRuntimeAuthority;
use crate::durability::authority::runtime_access::DurabilityAuthority;
use crate::durability::data::RelationalRecoveryStop;
use crate::runtime::RelationalRuntime;
use std::sync::Weak;

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
                runtime.services.origin_identity(),
                captured.description().clone(),
                admitted,
            )
        })
    }
}
fn require_row(
    runtime: &RelationalRuntime,
    index: &crate::history::commit::RelationalCommitIndexEntry,
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
impl DurabilityAuthority<'_> {
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

impl DurabilityRecoveryAuthority<'_> {
    pub fn restore_interim_checkpoint_for_codec_transition_with_authority(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        origin: &CapturedNativeInterimOrigin,
    ) -> Result<
        (
            crate::durability::data::RecoveryOutcome,
            RecoveredRelationalRuntimeAuthority,
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
            let checkpoint = decode_origin(
                origin,
                self.runtime.durability.medium().is_some(),
                &self.runtime.config.schema.registry,
                request,
            )?;
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
            crate::durability::data::RecoveryOutcome,
            RecoveredRelationalRuntimeAuthority,
        ),
        RelationalRecoveryStop,
    > {
        request_scope::run_recovery(request, || {
            let checkpoint = decode_origin(
                origin,
                self.runtime.durability.medium().is_some(),
                &self.runtime.config.schema.registry,
                request,
            )?;
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
            .bind_interim_original(row.original_row(), original, request)
            .map_err(RelationalRecoveryStop::AdmissionImage)?;
    }
    Ok(())
}
