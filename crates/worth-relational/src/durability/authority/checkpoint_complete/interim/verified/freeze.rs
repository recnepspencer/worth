//! Freeze one actual cut; no requestless standalone checkpoint intermediate.
use super::*;
use sha2::{Digest, Sha256};
impl FrozenMemoryOriginals {
    pub(in crate::durability::authority::checkpoint_complete::interim) fn capture(
        runtime: &crate::runtime::RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        if runtime.durability.medium().is_some() {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        crate::durability::artifact_payload_emission::request::account_work(request, 1)
            .map_err(stop)?;
        let serial = runtime
            .durability
            .checkpoint_selection()
            .next_interim_capture_serial()
            .map_err(Denial::AdmissionImage)?;
        let mut hash = Sha256::new();
        hash.update(b"worth.native.interim-capture/1");
        hash.update(runtime.services.runtime_instance_id().to_be_bytes());
        hash.update(serial.to_be_bytes());
        let coordinate = worth_foundational::CanonicalDigestId::new(hash.finalize().into());
        let binding = runtime.configuration_binding();
        let configuration = binding.operation();
        let routes = runtime.history.canonical_checkpoint_gate();
        routes
            .with_checkpoint_selection(|selection| {
                let current = FrozenCurrentCheckpointMetadata::capture(
                    runtime,
                    request,
                    &configuration.config.execution.runtime_name,
                    &configuration.config.history.main_branch,
                )?;
                let artifacts = runtime
                    .history
                    .commit_artifacts_for_request(request)
                    .map_err(array_stop)?;
                let same = match (artifacts.last(), selection.selected_receipt()) {
                    (None, None) => true,
                    (Some(artifact), Some(receipt)) => {
                        artifact.commit_id() == receipt.commit_id
                            && artifact.version_id() == receipt.version_id
                            && artifact.identity().authoring_branch() == &receipt.branch_id
                            && artifact.parentage().as_slice() == receipt.parents.as_slice()
                    }
                    (None, Some(_)) | (Some(_), None) => false,
                };
                if !same {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                let schemas =
                    match runtime.durability.checkpoint_selection().snapshot() {
                        None => None,
                        Some(directory) => {
                            let image = directory.admission_image.as_ref().ok_or(
                                Denial::AdmissionImage(super::super::Refusal::MissingCaptureOrigin),
                            )?;
                            match image.cut().scope() {
                                worth_foundational::facade::CheckpointAdmissionScope::Home {
                                    ..
                                } => return Err(Denial::OriginalAuthorityUnavailable),
                                worth_foundational::facade::CheckpointAdmissionScope::Interim {
                                    ..
                                } => {}
                            }
                            let directory = runtime
                                .durability
                                .checkpoint_selection()
                                .require_interim_image(image)
                                .map_err(Denial::AdmissionImage)?;
                            Some(
                                directory
                                    .source_schemas
                                    .get()
                                    .cloned()
                                    .ok_or(Denial::OriginalAuthorityUnavailable)?,
                            )
                        }
                    };
                for artifact in artifacts.iter() {
                    if let Some((entry, source)) = artifact.envelope_residency().source_placement()
                    {
                        let row = runtime
                            .durability
                            .checkpoint_selection()
                            .admitted_commit(&entry)
                            .map_err(Denial::AdmissionImage)?
                            .ok_or(Denial::OriginalAuthorityUnavailable)?;
                        if !runtime
                            .durability
                            .checkpoint_selection()
                            .matches_admitted_source(&row, &source)
                        {
                            return Err(Denial::OriginalAuthorityUnavailable);
                        }
                    }
                }
                Ok(Self {
                    current,
                    artifacts,
                    schemas,
                    symbols: runtime.services.symbols.interner_snapshot(),
                    mirror: runtime.acquire_partition_edition(),
                    transitions: selection.covered_transitions(),
                    owner: runtime.state_binding(),
                    coordinate,
                })
            })
            .map_err(crate::durability::authority::checkpointing::checkpoint_admission_error)?
    }
}
