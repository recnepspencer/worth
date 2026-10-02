use super::super::{map_discovery_failure, CheckpointDiscovery, DiscoveryFailure};
use crate::entry::{
    PhysicalRecoveryCheckpointIntegrityDenial, PhysicalRecoveryLimitDimension,
    PhysicalRecoveryLimits,
};
use crate::integrity_ingress::{
    admit_observed_checkpoint_stream, CheckpointStreamAdmissionFailure,
    RecoveryIntegrityIngressRejection, RecoveryIntegrityIngressTrace,
};
use crate::progression::PhysicalRecoveryDiscoveryCounters;
use worth_store::physical_runtime::{
    BoundedRecoveryFilesystemDiscovery, PhysicalRecoveryReadAllocation,
};

mod allocation;
pub(super) use allocation::window_admission_failure;

#[cfg(test)]
mod tests;

pub(super) fn observe_checkpoint(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    limits: PhysicalRecoveryLimits,
    remaining_manifest_bytes: &mut u64,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    ingress_trace: &mut RecoveryIntegrityIngressTrace,
    allocation: &mut PhysicalRecoveryReadAllocation<'_>,
) -> Result<CheckpointDiscovery, DiscoveryFailure> {
    let declaration = limits.declaration();
    let artifact = allocation
        .read_checkpoint(discovery, declaration.observation_bytes)
        .map_err(|failure| {
            allocation::map_read_failure(failure, |failure| {
                map_discovery_failure(
                    failure,
                    PhysicalRecoveryLimitDimension::ObservationBytes,
                    PhysicalRecoveryLimitDimension::ObservationBytes,
                )
            })
        })?;
    let mut trace = RecoveryIntegrityIngressTrace::new();
    let checkpoint = match admit_observed_checkpoint_stream(
        artifact.observed(),
        discovery.store_identity(),
        declaration.dirty_frames,
        declaration.operation_bindings,
        &mut trace,
        allocation,
    ) {
        Ok(Some(projection)) => {
            let generation = projection.checkpoint.facts().source().root().generation();
            let source_root = allocation
                .read_checkpoint_source_root(discovery, generation, *remaining_manifest_bytes)
                .map_err(|failure| {
                    allocation::map_read_failure(failure, |failure| {
                        super::super::map_cumulative_discovery_failure(
                            failure,
                            PhysicalRecoveryLimitDimension::ManifestEntries,
                            PhysicalRecoveryLimitDimension::ManifestBytes,
                            declaration.manifest_bytes,
                            *remaining_manifest_bytes,
                        )
                    })
                    .with_integrity_trace(trace.clone())
                })?;
            let bytes = source_root
                .observed()
                .bytes()
                .map_or(0, |bytes| bytes.len() as u64);
            *remaining_manifest_bytes = remaining_manifest_bytes
                .checked_sub(bytes)
                .ok_or(crate::entry::PhysicalRecoveryBlockKind::DiscoveryLimit)?;
            CheckpointDiscovery::Admitted {
                projection,
                source_root,
            }
        }
        Ok(None) => CheckpointDiscovery::Absent(
            artifact
                .into_absent()
                .expect("absent ingress retains no present checkpoint bytes"),
        ),
        Err(CheckpointStreamAdmissionFailure::ParserBacking { requested, cause }) => {
            return Err(allocation::parser_failure(requested, cause).with_integrity_trace(trace));
        }
        Err(CheckpointStreamAdmissionFailure::ParserCapacity { requested, actual }) => {
            return Err(
                allocation::parser_capacity_failure(requested, actual).with_integrity_trace(trace)
            );
        }
        Err(CheckpointStreamAdmissionFailure::Backing(cause)) => {
            return Err(DiscoveryFailure::from(
                crate::entry::PhysicalRecoveryBlockKind::Checkpoint,
            )
            .with_root_protocol_denials(&[
                crate::entry::PhysicalRecoverySourceDenial::CheckpointBacking(cause),
            ])
            .with_integrity_trace(trace));
        }
        Err(CheckpointStreamAdmissionFailure::BindingDecodeBacking(cause)) => {
            return Err(allocation::binding_decode_failure(cause).with_integrity_trace(trace));
        }
        Err(CheckpointStreamAdmissionFailure::BindingBasisBacking(cause)) => {
            return Err(allocation::binding_basis_failure(cause).with_integrity_trace(trace));
        }
        Err(CheckpointStreamAdmissionFailure::Binding(cause)) => {
            return Err(DiscoveryFailure::from(
                crate::entry::PhysicalRecoveryBlockKind::Checkpoint,
            )
            .with_root_protocol_denials(&[
                crate::entry::PhysicalRecoverySourceDenial::CheckpointBinding(cause),
            ])
            .with_integrity_trace(trace));
        }
        Err(CheckpointStreamAdmissionFailure::AllocationRejected) => CheckpointDiscovery::Rejected(
            PhysicalRecoveryCheckpointIntegrityDenial::AllocationRejected,
        ),
        Err(CheckpointStreamAdmissionFailure::Integrity(rejection)) => {
            CheckpointDiscovery::Rejected(checkpoint_denial(rejection))
        }
        Err(CheckpointStreamAdmissionFailure::DirtyRecordLimit { observed, admitted }) => {
            CheckpointDiscovery::Rejected(
                PhysicalRecoveryCheckpointIntegrityDenial::DirtyRecordLimit { observed, admitted },
            )
        }
        Err(CheckpointStreamAdmissionFailure::BindingRecordLimit { observed, admitted }) => {
            CheckpointDiscovery::Rejected(
                PhysicalRecoveryCheckpointIntegrityDenial::BindingRecordLimit {
                    observed,
                    admitted,
                },
            )
        }
    };
    let ingress = trace.counters();
    ingress_trace.append(trace);
    counters.checkpoint_integrity_attempts = ingress.attempted;
    counters.checkpoint_integrity_admissions = ingress.admitted;
    counters.checkpoint_integrity_rejections = ingress.attempted - ingress.admitted;
    counters.checkpoint_owner_projections = ingress.owner_projection_entries;
    counters.checkpoint_owner_decoder_entries = ingress.owner_decoder_entries;
    Ok(checkpoint)
}

fn checkpoint_denial(
    rejection: RecoveryIntegrityIngressRejection,
) -> PhysicalRecoveryCheckpointIntegrityDenial {
    match rejection {
        RecoveryIntegrityIngressRejection::Integrity(rejection) => {
            PhysicalRecoveryCheckpointIntegrityDenial::Integrity(rejection)
        }
        RecoveryIntegrityIngressRejection::NonCanonicalEncoding => {
            PhysicalRecoveryCheckpointIntegrityDenial::NonCanonicalEncoding
        }
        RecoveryIntegrityIngressRejection::ScopeMismatch
        | RecoveryIntegrityIngressRejection::SourceRangeOutsideObservation => {
            PhysicalRecoveryCheckpointIntegrityDenial::ScopeMismatch
        }
        RecoveryIntegrityIngressRejection::SourceIncarnationMismatch
        | RecoveryIntegrityIngressRejection::Absent
        | RecoveryIntegrityIngressRejection::MissingBoundedArtifact
        | RecoveryIntegrityIngressRejection::ConflictingDuplication { .. } => {
            PhysicalRecoveryCheckpointIntegrityDenial::SourceIncarnationMismatch
        }
    }
}
