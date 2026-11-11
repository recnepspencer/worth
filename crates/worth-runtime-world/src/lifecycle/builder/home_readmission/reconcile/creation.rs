//! Acquire each created branch's actual associated fork before its cold installation.
use super::*;
use crate::durability::{WorldBranchCreatedRecord, WorldCreatedBranchOwnerEvidence as Owners};
use worth_relational::facade::{
    branch::RecordedRelationalComponentReference, mvcc::RelationalCallerAssociation,
    runtime::RelationalRuntime,
};
use worth_signal::facade::durability::SignalRecordedSourcePoint;

pub(super) fn install<D, I, E, Ctx, T>(
    root: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    created: &WorldBranchCreatedRecord,
    source: &crate::branch::ProductBranchObservation,
    before_native: &RecordedRelationalComponentReference,
    source_point: &SignalRecordedSourcePoint,
    native: &RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    custody: &mut WorldNativeReadmissionCustody,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<crate::branch::ProductBranchObservation, Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let (native_original, signal_original) = match &created.owners {
        Owners::Reused => (None, None),
        Owners::Relational(native) => (Some(native), None),
        Owners::Signal(signal) => (None, Some(signal)),
        Owners::RelationalAndSignal { relational, signal } => (Some(relational), Some(signal)),
    };
    let acquired_native = match native_original {
        Some(original) => {
            let bytes = crate::durability::encode_association(request, &created.intent.association)
                .map_err(Denial::Medium)?;
            let association = RelationalCallerAssociation::from_bytes(bytes.elements())
                .map_err(|_| Denial::MalformedNativeAssociation)?;
            Some(
                authority
                    .admit_recorded_fork(
                        request,
                        native,
                        original.sequence,
                        &association,
                        &original.digest,
                        before_native,
                        &created.components.relational,
                    )
                    .map_err(|cause| Denial::NativeFork(Box::new(cause)))?,
            )
        }
        None => None,
    };
    let index = custody.creation_forks.len();
    custody.creation_forks.push(acquired_native);
    // Native custody is already in the owning refusal before Signal admission.
    let acquired_signal = match signal_original {
        Some(original) => Some(
            root.state
                .signal
                .authored()
                .ok_or(Denial::AuthoredSignalRequired)?
                .source_port()
                .readmit_source_point(original.point(), request)
                .map_err(|cause| Denial::SignalSource(Box::new(cause)))?,
        ),
        None => None,
    };
    custody.creation_signals.push(acquired_signal);
    let (observed, origin) = root.install_readmitted_created_branch(
        created,
        source,
        source_point,
        custody.creation_forks[index].as_ref(),
        custody.creation_signals[index].as_ref(),
        request,
    )?;
    custody.checkpoint_origins.push(origin);
    Ok(observed)
}

pub(super) fn retire_admissions(
    custody: &mut WorldNativeReadmissionCustody,
    native: &RelationalRuntime,
    request: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Denial> {
    for fork in &mut custody.creation_forks {
        if let Some(fork) = fork.take() {
            let (source, target, _original) = fork.into_parts();
            custody
                .fork_retirements
                .push(crate::basis::NativeBasisRetirement::new(
                    source.into_basis(),
                ));
            custody
                .fork_retirements
                .push(crate::basis::NativeBasisRetirement::new(
                    target.into_basis(),
                ));
        }
    }
    for original in &mut custody.fork_retirements {
        while !original
            .advance(request, native)
            .map_err(Denial::NativeForkRetirement)?
        {}
    }
    Ok(())
}
