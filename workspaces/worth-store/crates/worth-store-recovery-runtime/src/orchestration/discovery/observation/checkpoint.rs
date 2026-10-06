use super::super::{past_grant, refused_read, unread, CheckpointDiscovery, DiscoveryFailure};
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
use crate::orchestration::recovery_budget::{RecoveryAllowance, RecoveryReadBudget};
use crate::progression::PhysicalRecoveryDiscoveryCounters;
use worth_store::physical_runtime::{
    BoundedRecoveryFilesystemDiscovery, GrantedRead, GrantedReadStop,
    PhysicalRecoveryReadAllocation,
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
    manifest_bytes: &mut RecoveryReadBudget,
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
            let OversizedArtifact = allocation::refused(&declaration, failure, |failure| {
                refused_read(
                    failure,
                    stream,
                    &declaration,
                    PhysicalRecoveryLimitDimension::ObservationBytes,
                )
            })?;
            return Err(PhysicalRecoveryBlockKind::Checkpoint.into());
        }
    };
    let mut trace = RecoveryIntegrityIngressTrace::new();
    let checkpoint =
        match admit_observed_checkpoint_stream(
            artifact.observed(),
            discovery.store_identity(),
            declaration.dirty_frames,
            declaration.operation_bindings,
            &mut trace,
            allocation,
        ) {
            Ok(Some(projection)) => {
                let generation = projection.checkpoint.facts().source().root().generation();
                // The source root is one page of the declared format, and
                // spends recovery's manifest bytes.
                let source_root = match allocation
                    .read_checkpoint_source_root(
                        discovery,
                        record_format,
                        generation,
                        manifest_bytes.grant(),
                    )
                    .granted()
                {
                    Ok(source_root) => source_root,
                    Err(GrantedReadStop::PastGrant(overrun)) => {
                        return Err(past_grant(manifest_bytes, overrun).with_integrity_trace(trace));
                    }
                    Err(GrantedReadStop::Unread(failure)) => {
                        return Err(oversized_source_root(
                            generation,
                            allocation::refused(&declaration, failure, |failure| {
                                unread(failure, &declaration)
                            }),
                        )
                        .with_integrity_trace(trace));
                    }
                };
                manifest_bytes.charge(source_root.observed());
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
                return Err(allocation::parser_failure(&declaration, requested, cause)
                    .with_integrity_trace(trace));
            }
            Err(CheckpointStreamAdmissionFailure::ParserCapacity { requested, actual }) => {
                return Err(
                    allocation::parser_capacity_failure(&declaration, requested, actual)
                        .with_integrity_trace(trace),
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
                return Err(allocation::binding_decode_failure(&declaration, cause)
                    .with_integrity_trace(trace));
            }
            Err(CheckpointStreamAdmissionFailure::BindingBasisBacking(cause)) => {
                return Err(allocation::binding_basis_failure(&declaration, cause)
                    .with_integrity_trace(trace));
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
            Err(CheckpointStreamAdmissionFailure::AllocationRejected) => {
                CheckpointDiscovery::Rejected {
                    denial: PhysicalRecoveryCheckpointIntegrityDenial::AllocationRejected,
                    limit: None,
                }
            }
            Err(CheckpointStreamAdmissionFailure::Integrity(rejection)) => {
                CheckpointDiscovery::Rejected {
                    denial: checkpoint_denial(rejection),
                    limit: None,
                }
            }
            // The stream counts its records against the whole declared count.
            Err(CheckpointStreamAdmissionFailure::DirtyRecordLimit { observed, admitted }) => {
                CheckpointDiscovery::Rejected {
                    denial: PhysicalRecoveryCheckpointIntegrityDenial::DirtyRecordLimit,
                    limit: RecoveryAllowance::declared(
                        &declaration,
                        PhysicalRecoveryLimitDimension::DirtyFrames,
                    )
                    .beside(observed, admitted)
                    .map(Into::into),
                }
            }
            Err(CheckpointStreamAdmissionFailure::BindingRecordLimit { observed, admitted }) => {
                CheckpointDiscovery::Rejected {
                    denial: PhysicalRecoveryCheckpointIntegrityDenial::BindingRecordLimit,
                    limit: RecoveryAllowance::declared(
                        &declaration,
                        PhysicalRecoveryLimitDimension::OperationBindings,
                    )
                    .beside(observed, admitted)
                    .map(Into::into),
                }
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
