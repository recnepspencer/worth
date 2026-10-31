//! Admit all concrete Native custody before the original World root is installed.
use crate::durability::{
    RecoveredCustodyDisposition, WorldDurableMediumBinding, WorldNativeReadmissionCustody,
    WorldReadmissionDenial as Denial, WorldRecord, WorldRecoveredJournalState,
};
use crate::identity::readmission::{readmit_attempt, readmit_commit, readmit_custody};
use crate::lifecycle::owner::RuntimeWorldOwnerRoot;
use crate::recovery::{RecoveredWorldPublicationInputs, ReservedProductUnpublishedSlot};
use std::collections::BTreeMap;
use std::sync::Arc;
use worth_foundational::facade::DurableCompositePublicationAttemptIdentity;
use worth_relational::facade::durability::{
    ReadmittedRelationalAssociatedEffect, RecoveredRelationalRuntimeAuthority,
};

pub(super) fn reconcile<D, I, E, Ctx, T>(
    root: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    binding: &Arc<WorldDurableMediumBinding>,
    recovered: &WorldRecoveredJournalState,
    native: &worth_relational::facade::runtime::RelationalRuntime,
    authority: &RecoveredRelationalRuntimeAuthority,
    mut effects: BTreeMap<
        DurableCompositePublicationAttemptIdentity,
        ReadmittedRelationalAssociatedEffect,
    >,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Result<(), Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let Some(artifact) = recovered.branches.first() else {
        return Ok(());
    };
    let WorldRecord::Branch { components, .. } = &artifact.record else {
        unreachable!("readmitted branch frame")
    };
    let owner = root.owner_identity();
    let mut plans = Vec::new();
    for open in recovered.opens.values() {
        if matches!(
            recovered.dispositions.get(&open.association.attempt()),
            Some(RecoveredCustodyDisposition::Released(_))
        ) {
            continue;
        }
        let ids = (
            readmit_attempt(owner, open.association.attempt()).ok_or(Denial::ForeignOwner)?,
            readmit_commit(owner, open.commit).ok_or(Denial::ForeignOwner)?,
            readmit_custody(owner, open.custody).ok_or(Denial::ForeignOwner)?,
        );
        let slot: ReservedProductUnpublishedSlot = root
            .state
            .recovery
            .reserve_product_unpublished(owner)
            .map_err(|_| Denial::RecoveryCapacity)?;
        plans.push((open.clone(), ids, slot));
    }
    let signal = root
        .state
        .signal
        .basis_port()
        .readmit_recorded_initial_empty_root(&components.signal)
        .map_err(Denial::SignalInitialRoot)?;
    let bridge = root
        .state
        .bridge
        .readmit_recorded_runtime_world_baseline(&components.bridge, &signal)
        .map_err(Denial::BridgeBaseline)?;
    let mut custody = WorldNativeReadmissionCustody::empty();
    custody.root = Some(
        authority
            .admit_recorded_component(execution, native, &components.relational)
            .map_err(|cause| Denial::NativeComponent(Box::new(cause)))?,
    );
    for (open, _, _) in &plans {
        let admitted = (|| {
            let Some(effect) = effects.remove(&open.association.attempt()) else {
                return Ok(None);
            };
            let after = authority
                .recorded_publication_successor(execution, native, &effect)
                .map_err(Denial::NativeSuccessor)?;
            authority
                .admit_recorded_publication(
                    execution,
                    native,
                    effect,
                    &components.relational,
                    &after,
                )
                .map(Some)
                .map_err(|cause| Denial::NativePublication(Box::new(cause)))
        })();
        match admitted {
            Ok(admitted) => custody.publications.push(admitted),
            Err(cause) => return Err(custody.refuse(cause)),
        }
    }
    let before = custody
        .root
        .take()
        .expect("actual root admission retained until installation");
    let expected = match root.install_readmitted_root(artifact, before, signal, bridge, execution) {
        Ok(expected) => expected,
        Err(cause) => return Err(custody.refuse(cause)),
    };
    // All owner admissions and capacities succeeded. Insertion moves the exact
    // concrete capabilities into the existing reserved catalog slots.
    for ((open, (attempt, commit, identity), slot), native) in
        plans.into_iter().zip(custody.publications)
    {
        slot.install_recovered(RecoveredWorldPublicationInputs {
            identity,
            attempt,
            commit,
            open,
            expected: expected.clone(),
            binding: binding.clone(),
            native,
        });
    }
    Ok(())
}
