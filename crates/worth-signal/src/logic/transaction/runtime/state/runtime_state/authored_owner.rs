//! Rebuild every recovered branch, then move its cold state into the one owner.
use super::super::branching::{
    BranchManager, BranchState, SignalOwnerPartition, SignalReadmittedCellSeed, SnapshotBranchState,
};
use super::SignalRuntime;
use crate::data::graph::SignalGraphInstallationBindings;
use crate::durability::readmission::SignalOwnerSourceReadmission;
use crate::durability::{
    SignalDurableMedium, SignalDurableOwner, SignalDurableTierVocabulary,
    SignalOwnerReadmissionDenial as D, SignalReadmittedConditionalContract,
    SignalRecordedIdentityContribution,
};
use crate::state::{SignalBranchHandle, SignalBranchId, SignalSnapshotId};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::ExecutionRequest;

impl<Dm, I, E, Ctx, T> SignalRuntime<Dm, I, E, Ctx, T>
where
    Dm: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn readmit_authored_owner(
        self: Box<Self>,
        medium: Arc<dyn SignalDurableMedium>,
        contribution: SignalRecordedIdentityContribution,
        vocabulary: SignalDurableTierVocabulary<T>,
        installation: &mut dyn crate::durability::SignalAuthoredOwnerInstallation,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalDurableOwner<Dm, I, E, Ctx, T>, D> {
        if self.owner_services.is_sealed() {
            return Err(D::ProgramAlreadyServing);
        }
        self.owner_service_issuance_capability()
            .map_err(D::OwnerServices)?;
        execution.consult().map_err(|cause| {
            D::Medium(crate::durability::SignalDurableMediumDenial::Request(cause))
        })?;
        let durable_admission = medium
            .owner_admission_registry()
            .admit(medium.binding(), self.branches.owner_runtime_instance_id())
            .map_err(D::Medium)?;
        self.readmit_authored_owner_with_admission(
            medium,
            contribution,
            vocabulary,
            installation,
            execution,
            durable_admission,
        )
    }
    pub(crate) fn readmit_authored_owner_with_admission(
        mut self: Box<Self>,
        medium: Arc<dyn SignalDurableMedium>,
        contribution: SignalRecordedIdentityContribution,
        vocabulary: SignalDurableTierVocabulary<T>,
        installation: &mut dyn crate::durability::SignalAuthoredOwnerInstallation,
        execution: &ExecutionRequest<'_, '_>,
        durable_admission: crate::durability::SignalDurableOwnerAdmission,
    ) -> Result<SignalDurableOwner<Dm, I, E, Ctx, T>, D> {
        let mut source =
            SignalOwnerSourceReadmission::capture(medium.as_ref(), contribution, execution)?;
        let conditional = self
            .graph
            .installed_runtime_policy()
            .conditional_evaluation_budget();
        let temporal = self
            .graph
            .installed_runtime_policy()
            .conditional_temporal_budget();
        let bindings = SignalGraphInstallationBindings::consume(std::mem::take(&mut self.graph));
        // Every body participates in identity resumption, including a body that
        // an original advance never selected. No derived bytes are read.
        for location in &source.definitions {
            let artifact = source.read_location(location, execution)?;
            let (graph, _) = bindings
                .readmit(execution, &artifact.authored_image())
                .map_err(D::Image)?
                .into_parts();
            source
                .frontiers
                .observe_graph(artifact.branch(), &graph, execution)?;
        }
        source.frontiers.check_issuable()?;
        let branches = std::mem::take(&mut source.branches);
        let catalog = branches
            .iter()
            .map(|(identity, branch)| {
                (
                    SignalBranchId(identity.ordinal()),
                    SignalBranchHandle {
                        id: SignalBranchId(identity.ordinal()),
                        name: branch.name.clone(),
                        parent_branch_id: branch
                            .parent
                            .as_ref()
                            .map(|parent| SignalBranchId(parent.branch().ordinal())),
                        head_snapshot_id: branch
                            .head
                            .descriptor()
                            .snapshot()
                            .map(|id| SignalSnapshotId(id.ordinal())),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut seeds = Vec::with_capacity(branches.len());
        let mut contracts = BTreeMap::new();
        for (identity, branch) in branches {
            let artifact = source.read_definition(branch.head.descriptor(), execution)?;
            let (mut graph, installed) = bindings
                .readmit(execution, &artifact.authored_image())
                .map_err(D::Image)?
                .into_parts();
            graph.install_recorded_node_issuance(source.frontiers.node_issuance_root(identity));
            graph
                .diagnostics_state_mut()
                .synchronize_branch_catalog(&catalog, SignalBranchId(identity.ordinal()));
            let config = crate::logic::transaction::runtime::config::SignalRuntimeConfig::readmit_authored_image(
                execution, &artifact.authored_runtime_image(), &graph, &vocabulary).map_err(D::Image)?;
            let mut occurrences = artifact
                .conditional_occurrences()
                .map(|row| (row.node(), row))
                .collect::<BTreeMap<_, _>>();
            let mut restored_contracts = Vec::with_capacity(installed.len());
            for contract in installed {
                let node = contract.node();
                let durable = worth_foundational::facade::DurableSignalNodeIdentity::from_recorded(
                    identity,
                    node.index(),
                    node.generation(),
                );
                let occurrence = occurrences
                    .remove(&durable)
                    .ok_or(D::ConditionalOccurrenceMismatch(durable))?;
                restored_contracts.push(SignalReadmittedConditionalContract::new(
                    occurrence, contract,
                ));
            }
            if let Some((node, _)) = occurrences.first_key_value() {
                return Err(D::ConditionalOccurrenceMismatch(*node));
            }
            installation
                .readmit_branch(
                    crate::durability::SignalAuthoredBranchInstallation::new(
                        &graph,
                        branch.head.descriptor().branch(),
                        &restored_contracts,
                        artifact.bridge_bytes(),
                    ),
                    execution,
                )
                .map_err(|_| D::InstallationRefused(identity))?;
            contracts.insert(identity, restored_contracts);
            let handle = catalog
                .get(&SignalBranchId(identity.ordinal()))
                .expect("live source builds one handle")
                .clone();
            let state = BranchState::readmit_authored(
                graph,
                config,
                &handle,
                self.checkpoint.policy().clone(),
            );
            seeds.push(SignalReadmittedCellSeed {
                handle,
                state,
                point: branch.head,
                restored: branch.restored,
            });
        }
        let mut snapshots = BTreeMap::new();
        for (snapshot, saved) in &source.snapshots {
            let descriptor = &saved.descriptor;
            let artifact = source.read_definition(descriptor, execution)?;
            let (graph, _) = bindings
                .readmit(execution, &artifact.authored_image())
                .map_err(D::Image)?
                .into_parts();
            let config = crate::logic::transaction::runtime::config::SignalRuntimeConfig::readmit_authored_image(
                execution, &artifact.authored_runtime_image(), &graph, &vocabulary).map_err(D::Image)?;
            let handle = SignalBranchHandle {
                id: SignalBranchId(descriptor.branch().ordinal()),
                name: saved.name.clone(),
                parent_branch_id: saved
                    .parent
                    .as_ref()
                    .map(|parent| SignalBranchId(parent.branch().ordinal())),
                head_snapshot_id: Some(SignalSnapshotId(snapshot.ordinal())),
            };
            let state = BranchState::readmit_authored(
                graph,
                config,
                &handle,
                self.checkpoint.policy().clone(),
            );
            snapshots.insert(
                (handle.id, SignalSnapshotId(snapshot.ordinal())),
                SnapshotBranchState::from_branch_state(&state),
            );
        }
        let runtime_id = self.branches.owner_runtime_instance_id();
        let partition = SignalOwnerPartition::from_readmitted(
            runtime_id,
            seeds,
            snapshots,
            source.frontiers.next_branch()?,
            source.frontiers.next_snapshot()?,
            durable_admission,
        )?;
        self.discard_consumed_program_projection();
        self.branches = BranchManager::with_live_catalog(Default::default(), runtime_id);
        self.owner_services.seal(
            partition,
            self.schema_registry.registry_digest().to_owned(),
            conditional,
            temporal,
        );
        let owner = self
            .owner_services
            .downgrade_owner()
            .expect("readmission seals one owner");
        SignalDurableOwner::from_readmitted(
            self,
            source,
            medium,
            owner,
            Arc::new(vocabulary),
            contracts,
            execution,
        )
    }
    fn discard_consumed_program_projection(&mut self) {
        self.config = Default::default();
        self.checkpoint =
            crate::logic::checkpoint::CheckpointRuntime::new(self.checkpoint.policy().clone());
        self.resource = Default::default();
        self.temporal = Default::default();
        self.telemetry = Default::default();
    }
}
