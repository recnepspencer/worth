use super::{PhysicalRootPublicationTransition, PhysicalRootPublicationWorkPort};
use crate::physical_runtime::durability::{DurableMaintenanceReceipt, PhysicalRetirementDenial};
use crate::physical_runtime::record_serving::PreparedPhysicalRootCandidate;
use crate::physical_runtime::{
    PhysicalEffectIdentity, PhysicalRootPublicationWorkAction, PhysicalRootPublicationWorkScope,
    PhysicalWorkEffectFate, PhysicalWorkSettlementEvidence,
};

/// The same root publication executor and transition gate serve maintenance;
/// the evidence is a durable exact release intent, never a fabricated group.
pub(in crate::physical_runtime) struct NamespaceDurableRetirementRoot {
    pub(super) candidate: PreparedPhysicalRootCandidate,
    pub(super) transition: PhysicalRootPublicationTransition,
    pub(super) receipt: DurableMaintenanceReceipt,
    pub(super) replacement: PhysicalEffectIdentity,
    pub(super) namespace: PhysicalEffectIdentity,
}

pub(in crate::physical_runtime) struct NamespaceDurableManifestResidueRoot {
    pub(super) candidate: PreparedPhysicalRootCandidate,
    pub(super) transition: PhysicalRootPublicationTransition,
    pub(super) receipt: DurableMaintenanceReceipt,
    pub(super) replacement: PhysicalEffectIdentity,
    pub(super) namespace: PhysicalEffectIdentity,
}

#[cfg(feature = "certification-test-authority")]
pub(in crate::physical_runtime) struct NamespaceDurableTierEpochRoot {
    pub(super) candidate: PreparedPhysicalRootCandidate,
    pub(super) transition: PhysicalRootPublicationTransition,
    pub(super) receipt: DurableMaintenanceReceipt,
    pub(super) replacement: PhysicalEffectIdentity,
    pub(super) namespace: PhysicalEffectIdentity,
}

#[cfg(feature = "certification-test-authority")]
pub(in crate::physical_runtime) fn publish_tier_epoch_candidate(
    candidate: PreparedPhysicalRootCandidate,
    mut transition: PhysicalRootPublicationTransition,
    receipt: DurableMaintenanceReceipt,
    work: &PhysicalRootPublicationWorkPort,
) -> Result<NamespaceDurableTierEpochRoot, PhysicalRetirementDenial> {
    if transition.identity().tier_epoch_basis().is_none() {
        return Err(PhysicalRetirementDenial::WalPlan);
    }
    let identity = transition.identity();
    let run = || {
        for artifact in candidate.artifacts().iter().copied() {
            completed(
                work,
                identity,
                PhysicalRootPublicationWorkAction::SynchronizeCandidateArtifact { artifact },
            )?;
        }
        let replacement = completed(
            work,
            identity,
            PhysicalRootPublicationWorkAction::ReplaceBootstrapCatalog,
        )?;
        let namespace = completed(
            work,
            identity,
            PhysicalRootPublicationWorkAction::SynchronizeParentNamespace,
        )?;
        Ok::<_, PhysicalRetirementDenial>((replacement, namespace))
    };
    match run() {
        Ok((replacement, namespace)) => Ok(NamespaceDurableTierEpochRoot {
            candidate,
            transition,
            receipt,
            replacement,
            namespace,
        }),
        Err(denial) => {
            transition.require_inspection();
            Err(denial)
        }
    }
}

pub(in crate::physical_runtime) fn publish_manifest_residue_candidate(
    candidate: PreparedPhysicalRootCandidate,
    mut transition: PhysicalRootPublicationTransition,
    receipt: DurableMaintenanceReceipt,
    work: &PhysicalRootPublicationWorkPort,
) -> Result<NamespaceDurableManifestResidueRoot, PhysicalRetirementDenial> {
    if transition.identity().manifest_residue_basis().is_none() {
        return Err(PhysicalRetirementDenial::WalPlan);
    }
    let identity = transition.identity();
    let run = || {
        for artifact in candidate.artifacts().iter().copied() {
            completed(
                work,
                identity,
                PhysicalRootPublicationWorkAction::SynchronizeCandidateArtifact { artifact },
            )?;
        }
        let replacement = completed(
            work,
            identity,
            PhysicalRootPublicationWorkAction::ReplaceBootstrapCatalog,
        )?;
        let namespace = completed(
            work,
            identity,
            PhysicalRootPublicationWorkAction::SynchronizeParentNamespace,
        )?;
        Ok::<_, PhysicalRetirementDenial>((replacement, namespace))
    };
    match run() {
        Ok((replacement, namespace)) => Ok(NamespaceDurableManifestResidueRoot {
            candidate,
            transition,
            receipt,
            replacement,
            namespace,
        }),
        Err(denial) => {
            transition.require_inspection();
            Err(denial)
        }
    }
}

pub(in crate::physical_runtime) fn publish_retirement_candidate(
    candidate: PreparedPhysicalRootCandidate,
    mut transition: PhysicalRootPublicationTransition,
    receipt: DurableMaintenanceReceipt,
    work: &PhysicalRootPublicationWorkPort,
) -> Result<NamespaceDurableRetirementRoot, PhysicalRetirementDenial> {
    let identity = transition.identity();
    let run = || {
        for artifact in candidate.artifacts().iter().copied() {
            completed(
                work,
                identity,
                PhysicalRootPublicationWorkAction::SynchronizeCandidateArtifact { artifact },
            )?;
        }
        let replacement = completed(
            work,
            identity,
            PhysicalRootPublicationWorkAction::ReplaceBootstrapCatalog,
        )?;
        let namespace = completed(
            work,
            identity,
            PhysicalRootPublicationWorkAction::SynchronizeParentNamespace,
        )?;
        Ok::<_, PhysicalRetirementDenial>((replacement, namespace))
    };
    match run() {
        Ok((replacement, namespace)) => Ok(NamespaceDurableRetirementRoot {
            candidate,
            transition,
            receipt,
            replacement,
            namespace,
        }),
        Err(denial) => {
            transition.require_inspection();
            Err(denial)
        }
    }
}

fn completed(
    work: &PhysicalRootPublicationWorkPort,
    identity: super::PhysicalRootPublicationIdentity,
    action: PhysicalRootPublicationWorkAction,
) -> Result<PhysicalEffectIdentity, PhysicalRetirementDenial> {
    let scope = PhysicalRootPublicationWorkScope::new(identity, action)
        .ok_or(PhysicalRetirementDenial::Delete)?;
    let settlement = work
        .execute(scope)
        .map_err(|_| PhysicalRetirementDenial::Delete)?;
    if !matches!(
        settlement.evidence(),
        PhysicalWorkSettlementEvidence::PublicationEffect { .. }
    ) || settlement.evidence().fate() != PhysicalWorkEffectFate::PublicationCompleted
    {
        return Err(PhysicalRetirementDenial::Delete);
    }
    settlement
        .effect_identity()
        .ok_or(PhysicalRetirementDenial::Delete)
}
