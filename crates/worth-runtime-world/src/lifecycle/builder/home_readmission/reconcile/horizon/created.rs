//! Horizon readmission for surviving created branches whose source was disposed.
use super::HorizonContext;
use crate::branch::ProductBranchObservation;
use crate::durability::{
    WorldBranchCreatedRecord, WorldCreatedBranchOwnerEvidence as Owners,
    WorldNativeReadmissionCustody, WorldReadmissionDenial as Denial,
};
use crate::lifecycle::owner::{
    ReadmittedHorizonCommitSpec, ReadmittedHorizonInstall, RuntimeWorldOwnerRoot,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::RecordedRelationalComponentReference, durability::RecoveredRelationalRuntimeAuthority,
    mvcc::RelationalCallerAssociation, runtime::RelationalRuntime,
};
use worth_signal::facade::durability::{
    SignalAuthoredBranchChange, SignalReadmittedSourcePoint, SignalRecordedSourcePoint,
};

impl HorizonContext {
    #[allow(clippy::too_many_arguments)]
    pub(in super::super) fn install_surviving_created<D, I, E, Ctx, T>(
        &self,
        root: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
        created: &WorldBranchCreatedRecord,
        before_native: &RecordedRelationalComponentReference,
        source_signal: &SignalRecordedSourcePoint,
        native: &RelationalRuntime,
        authority: &RecoveredRelationalRuntimeAuthority,
        custody: &mut WorldNativeReadmissionCustody,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            ProductBranchObservation,
            RecordedRelationalComponentReference,
            SignalRecordedSourcePoint,
        ),
        Denial,
    >
    where
        D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
        I: Copy + Ord + Send + Sync + 'static,
        T: Copy + Ord + Send + Sync + 'static,
    {
        let (native_original, signal_original) = match &created.owners {
            Owners::Reused => return Err(Denial::CreationEvidenceMismatch),
            Owners::Relational(native) => (Some(native), None),
            Owners::Signal(signal) => (None, Some(signal)),
            Owners::RelationalAndSignal { relational, signal } => (Some(relational), Some(signal)),
        };
        let native_forked = native_original.is_some();
        let recovered_native = match native_original {
            Some(original) => {
                let bytes =
                    crate::durability::encode_association(execution, &created.intent.association)
                        .map_err(Denial::Medium)?;
                let association = RelationalCallerAssociation::from_bytes(bytes.elements())
                    .map_err(|_| Denial::MalformedNativeAssociation)?;
                authority
                    .admit_recorded_horizon_fork(
                        execution,
                        native,
                        original.sequence,
                        &association,
                        &original.digest,
                        before_native,
                        &created.components.relational,
                    )
                    .map_err(|cause| Denial::NativeFork(Box::new(cause)))?
            }
            None => {
                require_retained_native_predecessor(&created.components.relational, before_native)?;
                authority
                    .admit_recorded_component(execution, native, &created.components.relational)
                    .map_err(|cause| Denial::NativeComponent(Box::new(cause)))?
            }
        };
        let native_index = custody.roots.len();
        custody.roots.push(Some(recovered_native));
        let signal_port = root
            .state
            .signal
            .authored()
            .ok_or(Denial::AuthoredSignalRequired)?
            .source_port();
        let acquired_signal = match signal_original {
            Some(original) => {
                let actual = signal_port
                    .readmit_source_point(original.point(), execution)
                    .map_err(|cause| Denial::SignalSource(Box::new(cause)))?;
                validate_created_signal_fork(created, source_signal, &actual)?;
                Some(actual)
            }
            None => {
                if created.components.signal.point() != source_signal {
                    return Err(Denial::CreationEvidenceMismatch);
                }
                None
            }
        };
        let signal_index = custody.creation_signals.len();
        custody.creation_signals.push(acquired_signal);
        let fallback_signal = if custody.creation_signals[signal_index].is_none() {
            Some(
                signal_port
                    .readmit_source_point(created.components.signal.point(), execution)
                    .map_err(|cause| Denial::SignalSource(Box::new(cause)))?,
            )
        } else {
            None
        };
        let admitted_signal = custody.creation_signals[signal_index]
            .as_ref()
            .or(fallback_signal.as_ref())
            .expect("admitted signal source point");
        let native_basis = custody.roots[native_index]
            .take()
            .expect("actual acquired Native horizon basis")
            .into_basis();
        let bridge = root
            .state
            .bridge
            .as_ref()
            .expect("installed correspondence")
            .readmit_recorded_runtime_world_baseline(
                &created.components.bridge,
                admitted_signal.basis(),
            )
            .map_err(Denial::BridgeBaseline)?;
        let spliced_parent = self
            .ancestors
            .get(&created.intent.source.commit)
            .map(|(obs, dist)| (obs.selected_commit().clone(), dist.saturating_add(1)));
        let head = &created.intent.destination;
        let (observed, origin) = root.install_readmitted_horizon_commit(
            ReadmittedHorizonInstall {
                branch_name: head.branch.name(),
                incarnation: head.incarnation,
                generation: head.generation,
                commit: head.commit,
                is_root_branch: false,
                retirement_boundary: Some(created.intent.source.commit),
                branch_previous: None,
                spliced_parent,
                commit_spec: ReadmittedHorizonCommitSpec::BranchCreation {
                    original_parent: created.intent.source.commit,
                    attempt: created.intent.association.attempt(),
                    native_forked,
                    signal_change: custody.creation_signals[signal_index].as_ref(),
                },
                native: native_basis,
                signal: admitted_signal,
                bridge,
                expected_native: &created.components.relational,
                expected_signal: created.components.signal.point(),
            },
            execution,
        )?;
        custody.checkpoint_origins.push(origin);
        Ok((
            observed,
            created.components.relational.clone(),
            created.components.signal.point().clone(),
        ))
    }
}

#[cfg(test)]
pub(crate) fn require_retained_native_predecessor(
    claimed: &RecordedRelationalComponentReference,
    before_native: &RecordedRelationalComponentReference,
) -> Result<(), Denial> {
    if claimed != before_native {
        return Err(Denial::CreationEvidenceMismatch);
    }
    Ok(())
}

#[cfg(not(test))]
fn require_retained_native_predecessor(
    claimed: &RecordedRelationalComponentReference,
    before_native: &RecordedRelationalComponentReference,
) -> Result<(), Denial> {
    if claimed != before_native {
        return Err(Denial::CreationEvidenceMismatch);
    }
    Ok(())
}

fn validate_created_signal_fork(
    created: &WorldBranchCreatedRecord,
    source_signal: &SignalRecordedSourcePoint,
    actual: &SignalReadmittedSourcePoint,
) -> Result<(), Denial> {
    let intent = &created.intent;
    if actual.original().association() != Some(&intent.association)
        || actual.source_point() != created.components.signal.point()
    {
        return Err(Denial::CreationEvidenceMismatch);
    }
    let crate::branch::SignalBranchCreationPlan::ForkExact { target } = &intent.signal else {
        return Err(Denial::CreationEvidenceMismatch);
    };
    match actual.original().change() {
        SignalAuthoredBranchChange::Created {
            name,
            parent: Some(parent),
        } if name == target.as_str() && parent == source_signal.descriptor() => Ok(()),
        SignalAuthoredBranchChange::Created { .. }
        | SignalAuthoredBranchChange::Advanced
        | SignalAuthoredBranchChange::SnapshotCaptured
        | SignalAuthoredBranchChange::Restored
        | SignalAuthoredBranchChange::Retired => Err(Denial::CreationEvidenceMismatch),
    }
}
