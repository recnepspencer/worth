use super::super::{refused_read, CheckpointDiscovery, DiscoveryFailure};
use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryCheckpointIntegrityDenial,
    PhysicalRecoveryLimitDimension, PhysicalRecoveryLimits, PhysicalRecoveryRootProtocolArtifact,
    PhysicalRecoveryRootProtocolDenial, PhysicalRecoverySourceDenial,
};
use crate::integrity_ingress::{
    admit_observed_checkpoint_stream, CheckpointStreamAdmissionFailure,
    RecoveryIntegrityIngressRejection, RecoveryIntegrityIngressTrace,
};
use crate::orchestration::reader_limit::{OversizedArtifact, ReadCeiling};
use crate::progression::PhysicalRecoveryDiscoveryCounters;
use worth_store::physical_runtime::{
    BoundedRecoveryFilesystemDiscovery, PhysicalRecoveryReadAllocation,
};
use worth_store_physical_format::PhysicalRecordFormatDeclaration;

mod allocation;
pub(super) use allocation::window_admission_failure;

#[cfg(test)]
mod tests;

/// The checkpoint names a source root larger than a root manifest can be:
/// the same block a source root that fails its own admission raises.
fn oversized_source_root(
    generation: u64,
    refused: Result<OversizedArtifact, DiscoveryFailure>,
) -> DiscoveryFailure {
    match refused {
        Err(blocked) => blocked,
        Ok(OversizedArtifact) => DiscoveryFailure::from(PhysicalRecoveryBlockKind::Checkpoint)
            .with_root_protocol_denials(&[PhysicalRecoverySourceDenial::RootProtocol {
                artifact: PhysicalRecoveryRootProtocolArtifact::CheckpointSourceRoot { generation },
                denial: PhysicalRecoveryRootProtocolDenial::NonCanonicalEncoding,
            }]),
    }
}

pub(super) fn observe_checkpoint(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    limits: PhysicalRecoveryLimits,
    record_format: PhysicalRecordFormatDeclaration,
    remaining_manifest_bytes: &mut u64,
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    ingress_trace: &mut RecoveryIntegrityIngressTrace,
    allocation: &mut PhysicalRecoveryReadAllocation<'_>,
) -> Result<CheckpointDiscovery, DiscoveryFailure> {
    let declaration = limits.declaration();
    // Nothing declares the stream's length, so only the observation bytes
    // the caller admitted bound its read: those discovery has not yet read.
    let stream = ReadCeiling::of_budget_alone(
        declaration.observation_bytes,
        declaration
            .observation_bytes
            .saturating_sub(discovery.counters().bytes_read),
    );
    let artifact = match allocation.read_checkpoint(discovery, stream.requested()) {
        Ok(artifact) => artifact,
        Err(failure) => {
            let OversizedArtifact = allocation::refused(failure, |failure| {
                refused_read(
                    failure,
                    stream,
                    PhysicalRecoveryLimitDimension::ObservationBytes,
                )
            })?;
            return Err(PhysicalRecoveryBlockKind::Checkpoint.into());
        }
    };
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
            // A root manifest is one page of the declared format.
            let ceiling = ReadCeiling::within(
                u64::from(record_format.page_size().bytes()),
                declaration.manifest_bytes,
                *remaining_manifest_bytes,
            );
            let source_root = allocation
                .read_checkpoint_source_root(discovery, generation, ceiling.requested())
                .map_err(|failure| {
                    oversized_source_root(
                        generation,
                        allocation::refused(failure, |failure| {
                            refused_read(
                                failure,
                                ceiling,
                                PhysicalRecoveryLimitDimension::ManifestBytes,
                            )
                        }),
                    )
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
