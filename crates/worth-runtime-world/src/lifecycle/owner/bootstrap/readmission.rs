//! Install the original recorded root from concrete component-owner admissions.
use super::{preparation::ReadmittedRootPreparation, *};
use crate::durability::{DecodedWorldRecord, WorldReadmissionDenial as Denial, WorldRecord};
use crate::identity::readmission::{readmit_bootstrap, readmit_commit, readmit_incarnation};
use worth_relational::facade::durability::RecoveredRelationalBranchBasis;
use worth_runtime_bridge::facade::AdmittedRuntimeWorldCorrespondenceBasis;
use worth_signal::facade::durability::SignalReadmittedSourcePoint;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// The named factory obtains these three real admissions in owner order.
    /// Native is deliberately historical: comparing it to latest would replace
    /// the product's original predecessor after an unpublished Native effect.
    pub(crate) fn install_readmitted_root(
        &self,
        artifact: &DecodedWorldRecord,
        native: RecoveredRelationalBranchBasis,
        signal: SignalReadmittedSourcePoint,
        bridge: AdmittedRuntimeWorldCorrespondenceBasis,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            ProductBranchObservation,
            crate::durability::PreparedCheckpointOccurrence,
        ),
        Denial,
    > {
        let WorldRecord::Branch {
            head,
            bootstrap,
            components,
            ..
        } = &artifact.record
        else {
            unreachable!("owner readmission selected the original Branch frame")
        };
        let creation = crate::branch::ProductBranchCreationIntent::named(head.branch.name())
            .map_err(|_| Denial::BranchLineage)?;
        let intent = RuntimeWorldBootstrapIntent::recovered(
            creation,
            native,
            signal.basis().clone(),
            bridge,
        );
        let basis = crate::basis::admit_validated(
            &self
                .state
                .identities
                .lock()
                .unwrap_or_else(|e| e.into_inner()),
            intent.relational_basis().clone(),
            intent.signal_basis().clone(),
            intent.correspondence_basis().clone(),
        );
        let retained = basis.clone();
        let result = (|| {
            let original = self
                .state
                .relational_services()
                .basis_port()
                .export_recorded_retained_component(execution, basis.relational_basis())
                .map_err(Denial::NativeRecorded)?;
            if original != components.relational
                || signal.source_point() != components.signal.point()
            {
                return Err(Denial::BranchLineage);
            }
            let owner = self.owner_identity();
            let readmitted = ReadmittedRootPreparation {
                bootstrap: readmit_bootstrap(owner, *bootstrap).ok_or(Denial::ForeignOwner)?,
                branch: crate::identity::ProductBranchIdentity::issued(
                    owner,
                    intent.creation().name().clone(),
                ),
                incarnation: readmit_incarnation(owner, head.incarnation)
                    .ok_or(Denial::ForeignOwner)?,
                commit: readmit_commit(owner, head.commit).ok_or(Denial::ForeignOwner)?,
                basis,
            };
            let origin = crate::durability::PreparedCheckpointOccurrence::prepare(execution)
                .map_err(Denial::Medium)?;
            let mut prepared = self
                .prepare_readmitted_root(intent, readmitted, origin)
                .map_err(Denial::RootAdmission)?;
            // The branch frame was already selected through actual Home
            // recovery; installing its SAME identities performs no new I/O.
            let performed = prepared.install().map_err(Denial::RootAdmission)?;
            let origin = prepared
                .checkpoint_origin
                .take()
                .expect("same prepared admitted origin obligation");
            *self
                .state
                .bootstrap
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = RuntimeWorldBootstrapState::Performed;
            Ok((performed.product_branch().clone(), origin))
        })();
        result.map_err(|cause| {
            Denial::RootPreparation(Box::new(crate::durability::RefusedWorldRootReadmission {
                cause: Box::new(cause),
                basis: retained,
            }))
        })
    }
}

pub(crate) enum ReadmittedHorizonCommitSpec<'a> {
    Publication {
        original_parent: worth_foundational::facade::DurableCompositeCommitIdentity,
        attempt: worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
        native_commit: Option<worth_relational::facade::history::RelationalCommitIdentity>,
        signal_change: Option<&'a SignalReadmittedSourcePoint>,
    },
    BranchCreation {
        original_parent: worth_foundational::facade::DurableCompositeCommitIdentity,
        attempt: worth_foundational::facade::DurableCompositePublicationAttemptIdentity,
        native_forked: bool,
        signal_change: Option<&'a SignalReadmittedSourcePoint>,
    },
}

pub(crate) struct ReadmittedHorizonInstall<'a> {
    pub(crate) branch_name: &'a str,
    pub(crate) incarnation: worth_foundational::facade::DurableProductBranchIncarnation,
    pub(crate) generation: worth_foundational::facade::DurableProductReferenceGeneration,
    pub(crate) commit: worth_foundational::facade::DurableCompositeCommitIdentity,
    pub(crate) is_root_branch: bool,
    pub(crate) retirement_boundary:
        Option<worth_foundational::facade::DurableCompositeCommitIdentity>,
    pub(crate) branch_previous: Option<&'a ProductBranchReferenceSnapshot>,
    pub(crate) spliced_parent: Option<(crate::identity::CompositeCommitIdentity, usize)>,
    pub(crate) commit_spec: ReadmittedHorizonCommitSpec<'a>,
    pub(crate) native: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
    pub(crate) signal: &'a SignalReadmittedSourcePoint,
    pub(crate) bridge: AdmittedRuntimeWorldCorrespondenceBasis,
    pub(crate) expected_native:
        &'a worth_relational::facade::branch::RecordedRelationalComponentReference,
    pub(crate) expected_signal: &'a worth_signal::facade::durability::SignalRecordedSourcePoint,
}

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(crate) fn install_readmitted_horizon_commit(
        &self,
        install: ReadmittedHorizonInstall<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            ProductBranchObservation,
            crate::durability::PreparedCheckpointOccurrence,
        ),
        Denial,
    > {
        let basis = crate::basis::admit_validated(
            &self
                .state
                .identities
                .lock()
                .unwrap_or_else(|e| e.into_inner()),
            install.native,
            install.signal.basis().clone(),
            install.bridge,
        );
        let retained = basis.clone();
        let result = (|| {
            let original = self
                .state
                .relational_services()
                .basis_port()
                .export_recorded_retained_component(execution, basis.relational_basis())
                .map_err(Denial::NativeRecorded)?;
            if &original != install.expected_native
                || install.signal.source_point() != install.expected_signal
            {
                return Err(Denial::BranchLineage);
            }
            let owner = self.owner_identity();
            let name = crate::branch::ProductBranchName::try_new(install.branch_name)
                .map_err(|_| Denial::BranchLineage)?;
            let branch = crate::identity::ProductBranchIdentity::issued(owner, name.clone());
            let lifecycle =
                readmit_incarnation(owner, install.incarnation).ok_or(Denial::ForeignOwner)?;
            let commit_id = readmit_commit(owner, install.commit).ok_or(Denial::ForeignOwner)?;
            let commit_arc = match install.commit_spec {
                ReadmittedHorizonCommitSpec::Publication {
                    original_parent,
                    attempt,
                    native_commit,
                    signal_change,
                } => crate::history::CompositeRuntimeWorldCommit::from_readmitted_horizon_publication(
                    execution,
                    commit_id.clone(),
                    readmit_commit(owner, original_parent).ok_or(Denial::ForeignOwner)?,
                    basis.clone(),
                    crate::identity::readmission::readmit_attempt(owner, attempt)
                        .ok_or(Denial::ForeignOwner)?,
                    native_commit,
                    signal_change,
                )
                .map_err(|_| Denial::BranchLineage)?,
                ReadmittedHorizonCommitSpec::BranchCreation {
                    original_parent,
                    attempt,
                    native_forked,
                    signal_change,
                } => crate::history::CompositeRuntimeWorldCommit::from_readmitted_horizon_branch_creation(
                    execution,
                    commit_id.clone(),
                    readmit_commit(owner, original_parent).ok_or(Denial::ForeignOwner)?,
                    basis.clone(),
                    crate::identity::readmission::readmit_attempt(owner, attempt)
                        .ok_or(Denial::ForeignOwner)?,
                    native_forked,
                    signal_change,
                )
                .map_err(|_| Denial::CreationEvidenceMismatch)?,
            };
            let head_pin = self
                .state
                .retention
                .issue_product_head(&basis)
                .map_err(|_| Denial::RecoveryCapacity)?;
            let pins = head_pin
                .fork_history(&basis)
                .map_err(|_| Denial::RecoveryCapacity)?;
            let observation_capacity = self
                .state
                .retention
                .reserve_observation()
                .map_err(|_| Denial::RecoveryCapacity)?;
            let history = self
                .state
                .history
                .install_readmitted_horizon_commit(commit_arc.clone(), pins, install.spliced_parent)
                .map_err(Denial::HistoryAdmission)?;
            let snapshot = ProductBranchReferenceSnapshot::owner_issued(
                owner,
                branch.clone(),
                lifecycle,
                crate::identity::ProductBranchReferenceGeneration::readmitted(
                    install.generation.get(),
                ),
                commit_arc.clone(),
            )
            .map_err(|_| Denial::BranchLineage)?;
            let protection =
                ProductBranchHeadProtection::bootstrap_issued(snapshot.clone(), head_pin, history)
                    .map_err(|_| Denial::BranchLineage)?;
            if let Some(cell) = self.state.branches.branch_cell_by_lifecycle(lifecycle) {
                let before = install.branch_previous.ok_or(Denial::BranchLineage)?;
                cell.restore_readmitted_horizon(before, protection)
                    .map_err(|_| Denial::BranchLineage)?;
            } else {
                let mut cell = Some(
                    ProductBranchReferenceCell::new(protection)
                        .map_err(|_| Denial::BranchLineage)?,
                );
                if install.is_root_branch {
                    let mut reservation = self
                        .state
                        .branches
                        .reserve_root(owner)
                        .map_err(|_| Denial::RecoveryCapacity)?;
                    reservation
                        .install_root_borrowed(name, branch, lifecycle, &mut cell)
                        .map_err(|_| Denial::BranchLineage)?;
                } else {
                    let boundary = install
                        .retirement_boundary
                        .map(|id| readmit_commit(owner, id).ok_or(Denial::ForeignOwner))
                        .transpose()?;
                    let mut reservation = self
                        .state
                        .branches
                        .reserve_branch(owner, name.clone())
                        .map_err(|_| Denial::RecoveryCapacity)?;
                    reservation
                        .install_horizon_branch_borrowed(
                            name, branch, lifecycle, &mut cell, boundary,
                        )
                        .map_err(|_| Denial::BranchLineage)?;
                }
            }
            *self
                .state
                .bootstrap
                .lock()
                .unwrap_or_else(|e| e.into_inner()) = RuntimeWorldBootstrapState::Performed;
            let observation = self
                .issue_reused_observation(snapshot, commit_arc, observation_capacity)
                .map_err(|_| Denial::RecoveryCapacity)?;
            let mut origin = crate::durability::PreparedCheckpointOccurrence::prepare(execution)
                .map_err(Denial::Medium)?;
            origin.bind(
                observation.selected_commit().ordinal(),
                observation.basis(),
                observation
                    .fork_checkpoint_history()
                    .map_err(|_| Denial::RecoveryCapacity)?,
            );
            Ok((observation, origin))
        })();
        result.map_err(|cause| {
            Denial::RootPreparation(Box::new(crate::durability::RefusedWorldRootReadmission {
                cause: Box::new(cause),
                basis: retained,
            }))
        })
    }
}
