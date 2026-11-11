//! An exact captured fork becomes authored definition and Created owner frames.
use super::*;
use crate::branch::owner_services::owner::fork_reservation::SignalPreparedOwnedFork;
use crate::durability::{
    SignalAuthoredBranchChange, SignalDurableBranchDescriptor, SignalDurableForkPublication,
    SignalForkPublicationContext,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    CompositeAttemptAssociation, DurableSignalCellIncarnation,
    DurableSignalConditionalDefinitionOccurrence, DurableSignalGeneration,
    DurableSignalNodeIdentity,
};

impl<D, I, T> SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(super) fn prepare_fork_frames(
        &self,
        original: SignalBranchAdvanceIdentity,
        admission: DurableSignalAdmissionIdentity,
        source: SignalDurableBranchDescriptor,
        cell: DurableSignalCellIncarnation,
        mut prepared: SignalPreparedOwnedFork<D, I, T>,
        retention: SignalBranchAdmissionReservation,
        mut publication: Box<dyn SignalDurableForkPublication>,
        association: Option<CompositeAttemptAssociation>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<TaskPhase<D, I, T>, SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        let branch = cell.branch();
        let previous = self
            .source
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?
            .definition_record(&source)
            .map_err(Dn::Readmission)?;
        let previous = previous
            .read(self.medium.as_ref(), execution)
            .map_err(Dn::Readmission)?;
        let location =
            SignalAdvanceLocationReservation::reserve(execution).map_err(Dn::Readmission)?;
        let definition_location =
            SignalDefinitionLocationReservation::reserve(execution).map_err(Dn::Readmission)?;
        let count = previous.conditional_occurrences().count();
        let bytes = crate::data::retained_storage::RetainedStorageCharge::capacity::<
            DurableSignalConditionalDefinitionOccurrence,
        >(count)
        .map_err(|_| {
            Dn::Readmission(crate::durability::SignalOwnerReadmissionDenial::MetadataCapacity)
        })?
        .bytes();
        let _scratch = execution
            .reserve_retained_memory(bytes)
            .map_err(Dn::Request)?;
        let mut occurrences = Vec::with_capacity(count);
        let name = prepared.branch().name.clone();
        let observation = prepared.observation().clone();
        let state = prepared.state_mut();
        let graph = state.state().graph();
        let identity = {
            let mut source = self
                .source
                .lock()
                .map_err(|_| Dn::PublicationStatePoisoned)?;
            let identity = source
                .frontiers
                .issue_definition()
                .map_err(Dn::Readmission)?;
            source
                .frontiers
                .observe_graph(branch, graph, execution)
                .map_err(Dn::Readmission)?;
            for inherited in previous.conditional_occurrences() {
                let node = inherited.node();
                let occurrence = source
                    .frontiers
                    .issue_conditional()
                    .map_err(Dn::Readmission)?;
                occurrences.push(
                    DurableSignalConditionalDefinitionOccurrence::from_recorded(
                        DurableSignalNodeIdentity::from_recorded(
                            branch,
                            node.index(),
                            node.generation(),
                        ),
                        occurrence,
                    )
                    .map_err(|_| Dn::ConditionalDefinitionMismatch)?,
                );
            }
            identity
        };
        let image = graph
            .capture_authored_image(execution, branch)
            .map_err(Dn::Image)?;
        let runtime = state
            .state()
            .config()
            .capture_authored_image(graph, &self.vocabulary, branch, execution)
            .map_err(Dn::Image)?;
        let bridge = publication.prepare(
            execution,
            SignalForkPublicationContext {
                original,
                branch,
                observation: &observation,
                graph,
                predecessor: &previous,
                occurrences: &occurrences,
            },
        )?;
        let definition = SignalDefinitionTransitionArtifact::capture(
            execution,
            identity,
            branch,
            Some(source.definition()),
            &image,
            &runtime,
            &occurrences,
            &bridge,
        )
        .map_err(Dn::Frame)?;
        let descriptor = SignalDurableBranchDescriptor::from_recorded(
            DurableSignalGeneration::from_recorded(cell, 0),
            source.snapshot(),
            admission,
            definition.reference(),
        )
        .map_err(Dn::Frame)?;
        let artifact = SignalBranchAdvanceArtifact::capture(
            execution,
            original,
            descriptor,
            association,
            SignalAuthoredBranchChange::Created {
                name,
                parent: Some(source),
            },
        )
        .map_err(Dn::Frame)?;
        Ok(TaskPhase::Definition(Box::new(DefinitionPending {
            advance: ReadyPublication {
                prepared: PreparedSignalOwnerEffect::Forked(Box::new(prepared)),
                artifact,
                retention,
                location,
                publication: Some(
                    publication as Box<dyn crate::durability::SignalDurableProjectionPublication>,
                ),
            },
            definition,
            location: definition_location,
        })))
    }
}
