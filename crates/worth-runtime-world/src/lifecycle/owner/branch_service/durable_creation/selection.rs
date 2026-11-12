//! Materialize the exact destination and wire description before durable selection.
use super::*;
use crate::branch::{RelationalBranchCreationPlan as Native, SignalBranchCreationPlan as Signal};
use crate::durability::*;
impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn prepare_creation_selection(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        custody: &mut ActiveAttemptCustody,
        body: &mut WorldDurableCreationCustody,
    ) -> Result<(), RuntimeWorldBranchAdmissionDenial> {
        use RuntimeWorldBranchAdmissionDenial as Denial;
        let reuse = matches!(body.plan.relational(), Native::ReuseExact)
            && matches!(body.plan.signal(), Signal::ReuseExact);
        if !custody.creation_prepared() {
            if reuse {
                let commit = body.source.snapshot().shared_commit();
                let (cell, snapshot) =
                    self.issue_reused_head(body.branch.clone(), body.incarnation, commit.clone())?;
                custody
                    .issue_creation_observation(
                        &self.state.retention,
                        &self.state.history,
                        snapshot,
                    )
                    .map_err(|_| Denial::OwnerUnavailable)?;
                custody.adopt_reused_creation_destination(commit, cell);
            } else {
                let basis = match custody.durable_creation_commit() {
                    Some(original) => original.basis().clone(),
                    None => self
                        .issue_successor_basis_from_progress(&body.progress, body.source.basis()),
                };
                if !self.successor_correspondence_is_valid(&basis) {
                    return Err(Denial::OwnerUnavailable);
                }
                let (_, results) = body
                    .progress
                    .retained_image()
                    .into_ready_results()
                    .map_err(|_| Denial::OwnerUnavailable)?;
                assert!(results.matches_creation_plan(&body.plan));
                let commit = custody.prepare_commit(basis, &results);
                custody
                    .bind_publication_pins(commit.basis())
                    .map_err(map_retention_denial)?;
                custody.install_creation_history(&commit);
                let snapshot = ProductBranchReferenceSnapshot::owner_issued(
                    self.owner_identity(),
                    body.branch.clone(),
                    body.incarnation,
                    ProductBranchReferenceGeneration::initial(),
                    commit,
                )
                .map_err(|_| Denial::OwnerUnavailable)?;
                custody
                    .issue_creation_observation(
                        &self.state.retention,
                        &self.state.history,
                        snapshot.clone(),
                    )
                    .map_err(|_| Denial::OwnerUnavailable)?;
                custody.prepare_creation_cell(snapshot);
            }
        }
        let snapshot = custody.creation_snapshot();
        let basis = snapshot.commit().basis();
        let native = self
            .state
            .relational_services()
            .basis_port()
            .export_recorded_retained_component(execution, basis.relational_basis())
            .map_err(|_| Denial::OwnerUnavailable)?;
        let signal = match &self.state.signal {
            crate::lifecycle::signal_services::RuntimeWorldSignalServices::Authored(actual) => {
                actual
                    .recorded_current_basis(basis.signal_basis())
                    .map_err(|_| Denial::OwnerUnavailable)?
            }
            crate::lifecycle::signal_services::RuntimeWorldSignalServices::Memory { .. } => {
                unreachable!("Home profile")
            }
        };
        let signal = WorldSignalEvidence::from_point(signal, execution)
            .map_err(|_| Denial::CapacityExhausted)?;
        let bridge = self
            .state
            .bridge
            .as_ref()
            .expect("composition")
            .export_recorded_runtime_world_baseline_in(execution, basis.correspondence_basis())
            .map_err(|_| Denial::OwnerUnavailable)?;
        let native_evidence =
            body.native_original
                .as_ref()
                .map(|actual| WorldRelationalForkEvidence {
                    sequence: actual.positioned().sequence(),
                    digest: *actual.artifact_digest(),
                });
        let owners = match (native_evidence, body.plan.signal()) {
            (None, Signal::ReuseExact) => WorldCreatedBranchOwnerEvidence::Reused,
            (Some(actual), Signal::ReuseExact) => {
                WorldCreatedBranchOwnerEvidence::Relational(actual)
            }
            (None, Signal::ForkExact { .. }) => {
                WorldCreatedBranchOwnerEvidence::Signal(signal.clone())
            }
            (Some(relational), Signal::ForkExact { .. }) => {
                WorldCreatedBranchOwnerEvidence::RelationalAndSignal {
                    relational,
                    signal: signal.clone(),
                }
            }
        };
        let WorldRecord::CreationIntent(intent) = body.subject.record() else {
            unreachable!("original class")
        };
        let selected = WorldDurableArtifact::from_input(
            WorldArtifactInput::BranchCreated {
                intent: WorldCreationIntentInput::from_record(intent),
                components: RecordedWorldInitialComponents {
                    relational: native,
                    signal,
                    bridge,
                },
                owners,
                publishing_incarnation: body.binding.binding().incarnation(),
            },
            execution,
        )
        .map_err(|_| Denial::CapacityExhausted)?;
        let observation = custody.creation_checkpoint_observation();
        let origin = body.origin.as_mut().expect("prepaid recipe");
        if origin.needs_binding() {
            origin.bind(
                basis,
                observation
                    .fork_checkpoint_history()
                    .map_err(|_| Denial::CapacityExhausted)?,
            );
        }
        body.selected = Some(selected);
        Ok(())
    }
}
