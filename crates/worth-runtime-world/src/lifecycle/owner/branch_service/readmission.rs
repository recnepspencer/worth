//! Install selected branch births from actual recovered component owners.
use super::*;
use crate::durability::{WorldBranchCreatedRecord, WorldReadmissionDenial as Denial};
use crate::identity::readmission::{readmit_attempt, readmit_commit, readmit_incarnation};
use worth_relational::facade::durability::ReadmittedRelationalAssociatedFork;
use worth_signal::facade::durability::{
    SignalAuthoredBranchChange, SignalReadmittedSourcePoint, SignalRecordedSourcePoint,
};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn install_readmitted_created_branch(
        &self,
        created: &WorldBranchCreatedRecord,
        source: &ProductBranchObservation,
        source_signal: &SignalRecordedSourcePoint,
        native: Option<&ReadmittedRelationalAssociatedFork>,
        signal: Option<&SignalReadmittedSourcePoint>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            ProductBranchObservation,
            crate::durability::PreparedCheckpointOccurrence,
        ),
        Denial,
    > {
        let intent = &created.intent;
        if let Some(actual) = signal {
            if actual.original().association() != Some(&intent.association)
                || actual.source_point() != created.components.signal.point()
            {
                return Err(Denial::CreationEvidenceMismatch);
            }
            let crate::branch::SignalBranchCreationPlan::ForkExact { target } = &intent.signal
            else {
                return Err(Denial::CreationEvidenceMismatch);
            };
            match actual.original().change() {
                SignalAuthoredBranchChange::Created {
                    name,
                    parent: Some(parent),
                } if name == target.as_str() && parent == source_signal.descriptor() => {}
                SignalAuthoredBranchChange::Created { .. }
                | SignalAuthoredBranchChange::Advanced
                | SignalAuthoredBranchChange::SnapshotCaptured
                | SignalAuthoredBranchChange::Restored
                | SignalAuthoredBranchChange::Retired => {
                    return Err(Denial::CreationEvidenceMismatch)
                }
            }
        } else if created.components.signal.point() != source_signal {
            return Err(Denial::CreationEvidenceMismatch);
        }
        let native_basis = native.map_or_else(
            || source.basis().relational_basis(),
            |fork| fork.target_basis(),
        );
        let original = self
            .state
            .relational_services()
            .basis_port()
            .export_recorded_retained_component(request, native_basis)
            .map_err(Denial::NativeRecorded)?;
        if original != created.components.relational {
            return Err(Denial::CreationEvidenceMismatch);
        }
        let bridge = self
            .state
            .bridge
            .as_ref()
            .expect("installed correspondence");
        let correspondence = match signal {
            Some(actual) => bridge
                .readmit_recorded_runtime_world_baseline(&created.components.bridge, actual.basis())
                .map_err(Denial::BridgeBaseline)?,
            None => {
                let original = bridge
                    .export_recorded_runtime_world_baseline_in(
                        request,
                        source.basis().correspondence_basis(),
                    )
                    .map_err(Denial::BridgeBaseline)?;
                if original != created.components.bridge {
                    return Err(Denial::CreationEvidenceMismatch);
                }
                source.basis().correspondence_basis().clone()
            }
        };
        let basis = if native.is_none() && signal.is_none() {
            source.basis().clone()
        } else {
            crate::basis::admit_validated_for_request(
                request,
                &self.state.identities,
                native_basis.clone(),
                signal.map_or_else(
                    || source.basis().signal_basis().clone(),
                    |actual| actual.basis().clone(),
                ),
                correspondence,
            )
            .map_err(|cause| match cause {
                crate::basis::CompositeBasisBackingDenial::Request(cause) => {
                    Denial::Medium(crate::durability::WorldDurableMediumDenial::Request(cause))
                }
                crate::basis::CompositeBasisBackingDenial::CapacityOverflow => {
                    Denial::RecoveryCapacity
                }
            })?
        };
        let retained = basis.clone();
        (|| {
            let owner = self.owner_identity();
            let head = &intent.destination;
            let name = ProductBranchName::try_new(head.branch.name())
                .map_err(|_| Denial::BranchLineage)?;
            let branch = ProductBranchIdentity::issued(owner, name.clone());
            let incarnation =
                readmit_incarnation(owner, head.incarnation).ok_or(Denial::ForeignOwner)?;
            let mut reservation = self
                .state
                .branches
                .reserve_branch(owner, name)
                .map_err(|_| Denial::RecoveryCapacity)?;
            let observation_capacity = self
                .state
                .retention
                .reserve_observation()
                .map_err(|_| Denial::RecoveryCapacity)?;
            let commit = if native.is_none() && signal.is_none() {
                if source.selected_commit().ordinal() != head.commit.ordinal() {
                    return Err(Denial::CreationEvidenceMismatch);
                }
                source.snapshot().shared_commit()
            } else {
                crate::history::CompositeRuntimeWorldCommit::from_readmitted_branch_creation(
                    request,
                    readmit_commit(owner, head.commit).ok_or(Denial::ForeignOwner)?,
                    source.snapshot().commit(),
                    basis.clone(),
                    readmit_attempt(owner, intent.association.attempt())
                        .ok_or(Denial::ForeignOwner)?,
                    native,
                    signal,
                )
                .map_err(|cause| match cause {
                    crate::history::ReadmittedWorldCommitDenial::Request(cause) => {
                        Denial::Medium(crate::durability::WorldDurableMediumDenial::Request(cause))
                    }
                    crate::history::ReadmittedWorldCommitDenial::CapacityOverflow => {
                        Denial::RecoveryCapacity
                    }
                    crate::history::ReadmittedWorldCommitDenial::Native(cause) => {
                        Denial::NativeSuccessor(cause)
                    }
                    crate::history::ReadmittedWorldCommitDenial::Construction(
                        crate::history::CompositeCommitConstructionDenial::OwnerMismatch
                        | crate::history::CompositeCommitConstructionDenial::BasisMismatch,
                    ) => Denial::CreationEvidenceMismatch,
                })?
            };
            let (cell, snapshot) = if native.is_none() && signal.is_none() {
                self.issue_reused_head(branch.clone(), incarnation, commit.clone())
                    .map_err(|_| Denial::RecoveryCapacity)?
            } else {
                let mut capacity = self
                    .state
                    .history
                    .reserve_commit_capacity(commit.identity().clone(), commit.parent().clone())
                    .map_err(Denial::HistoryAdmission)?;
                let head_pin = self
                    .state
                    .retention
                    .issue_product_head(&basis)
                    .map_err(|_| Denial::RecoveryCapacity)?;
                let pins = head_pin
                    .fork_history(&basis)
                    .map_err(|_| Denial::RecoveryCapacity)?;
                let history = capacity
                    .try_install_product_head(commit.clone(), pins)
                    .map_err(Denial::HistoryAdmission)?;
                let snapshot = ProductBranchReferenceSnapshot::owner_issued(
                    owner,
                    branch.clone(),
                    incarnation,
                    ProductBranchReferenceGeneration::initial(),
                    commit.clone(),
                )
                .map_err(|_| Denial::BranchLineage)?;
                let protection = ProductBranchHeadProtection::bootstrap_issued(
                    snapshot.clone(),
                    head_pin,
                    history,
                )
                .map_err(|_| Denial::BranchLineage)?;
                (
                    ProductBranchReferenceCell::new(protection)
                        .map_err(|_| Denial::BranchLineage)?,
                    snapshot,
                )
            };
            let observation = self
                .issue_reused_observation(snapshot, commit, observation_capacity)
                .map_err(|_| Denial::RecoveryCapacity)?;
            let mut origin = crate::durability::PreparedCheckpointOccurrence::prepare(request)
                .map_err(Denial::Medium)?;
            origin.bind(
                observation.basis(),
                observation
                    .fork_checkpoint_history()
                    .map_err(|_| Denial::RecoveryCapacity)?,
            );
            reservation
                .install_readmitted_branch(source, branch, incarnation, &mut Some(cell))
                .map_err(|_| Denial::BranchLineage)?;
            Ok((observation, origin))
        })()
        .map_err(|cause| {
            Denial::RootPreparation(Box::new(crate::durability::RefusedWorldRootReadmission {
                cause: Box::new(cause),
                basis: retained,
            }))
        })
    }
}
