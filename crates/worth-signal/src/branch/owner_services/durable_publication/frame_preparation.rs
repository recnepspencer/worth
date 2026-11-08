//! Capture the authored successor before the original journal driver starts.
use super::*;
use crate::durability::{SignalAuthoredBranchChange, SignalDurableBranchDescriptor};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurableSignalGeneration;
impl<D, I, T> SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(super) fn prepare_frames(
        &self,
        original: SignalBranchAdvanceIdentity,
        admission: DurableSignalAdmissionIdentity,
        prepared: SignalPreparedDurableAdvance<D, I, T>,
        retention: SignalBranchAdmissionReservation,
        mut publication: Option<Box<dyn crate::durability::SignalDurableDefinitionPublication>>,
        association: Option<worth_foundational::facade::CompositeAttemptAssociation>,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<TaskPhase<D, I, T>, SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        let branch = prepared.predecessor.branch();
        let location =
            SignalAdvanceLocationReservation::reserve(execution).map_err(Dn::Readmission)?;
        if prepared.source_changed {
            let (identity, previous_source) = {
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
                    .observe_graph(branch, prepared.state.graph(), execution)
                    .map_err(Dn::Readmission)?;
                let previous = source
                    .definition_record(&prepared.predecessor)
                    .map_err(Dn::Readmission)?;
                (identity, previous)
            };
            let previous = previous_source
                .read(self.medium.as_ref(), execution)
                .map_err(Dn::Readmission)?;
            let image = prepared
                .state
                .graph()
                .capture_authored_image(execution, branch)
                .map_err(Dn::Image)?;
            let runtime = prepared
                .state
                .config()
                .capture_authored_image(prepared.state.graph(), &self.vocabulary, branch, execution)
                .map_err(Dn::Image)?;
            let mut occurrences = previous.conditional_occurrences().collect::<Vec<_>>();
            let bridge = if let Some(conditional) = prepared.conditional.as_ref() {
                let contract = &conditional.contract;
                let mut source = self
                    .source
                    .lock()
                    .map_err(|_| Dn::PublicationStatePoisoned)?;
                let occurrence = source
                    .frontiers
                    .issue_conditional()
                    .map_err(Dn::Readmission)?;
                drop(source);
                let node = contract.node();
                let occurrence = worth_foundational::facade::DurableSignalConditionalDefinitionOccurrence::from_recorded(
                    worth_foundational::facade::DurableSignalNodeIdentity::from_recorded(branch, node.index(), node.generation()), occurrence,
                ).map_err(|_| Dn::ConditionalDefinitionMismatch)?;
                let row = occurrences
                    .iter_mut()
                    .find(|old| old.node() == occurrence.node())
                    .ok_or(Dn::ConditionalDefinitionMismatch)?;
                *row = occurrence;
                publication
                    .as_mut()
                    .ok_or(Dn::ConditionalDefinitionPublicationRequired)?
                    .prepare(
                        execution,
                        crate::durability::SignalDefinitionPublicationContext {
                            original,
                            identity,
                            occurrence,
                            predecessor: &previous,
                            contract,
                            custody: &conditional.custody,
                        },
                    )?
            } else {
                previous.bridge_section()
            };
            let definition = SignalDefinitionTransitionArtifact::capture(
                execution,
                identity,
                branch,
                Some(prepared.predecessor.definition()),
                &image,
                &runtime,
                &occurrences,
                &bridge,
            )
            .map_err(Dn::Frame)?;
            let descriptor = SignalDurableBranchDescriptor::from_recorded(
                DurableSignalGeneration::from_recorded(
                    prepared.predecessor.cell(),
                    prepared.generation,
                ),
                prepared.predecessor.snapshot(),
                admission,
                definition.reference(),
            )
            .map_err(Dn::Frame)?;
            let artifact = SignalBranchAdvanceArtifact::capture(
                execution,
                original,
                descriptor,
                association,
                SignalAuthoredBranchChange::Advanced,
            )
            .map_err(Dn::Frame)?;
            Ok(TaskPhase::Definition(Box::new(DefinitionPending {
                advance: ReadyPublication {
                    prepared: PreparedSignalOwnerEffect::Advanced(Box::new(prepared)),
                    artifact,
                    retention,
                    location,
                    publication: publication.map(|publication| {
                        publication
                            as Box<dyn crate::durability::SignalDurableProjectionPublication>
                    }),
                },
                definition,
                location: SignalDefinitionLocationReservation::reserve(execution)
                    .map_err(Dn::Readmission)?,
            })))
        } else {
            let definition = self
                .source
                .lock()
                .map_err(|_| Dn::PublicationStatePoisoned)?
                .definition_record(&prepared.predecessor)
                .map_err(Dn::Readmission)?;
            let descriptor = SignalDurableBranchDescriptor::from_recorded(
                DurableSignalGeneration::from_recorded(
                    prepared.predecessor.cell(),
                    prepared.generation,
                ),
                prepared.predecessor.snapshot(),
                admission,
                definition.reference(),
            )
            .map_err(Dn::Frame)?;
            let artifact = SignalBranchAdvanceArtifact::capture(
                execution,
                original,
                descriptor,
                association,
                SignalAuthoredBranchChange::Advanced,
            )
            .map_err(Dn::Frame)?;
            Ok(TaskPhase::Branch(Box::new(BranchPending {
                advance: ReadyPublication {
                    prepared: PreparedSignalOwnerEffect::Advanced(Box::new(prepared)),
                    artifact,
                    retention,
                    location,
                    publication: publication.map(|publication| {
                        publication
                            as Box<dyn crate::durability::SignalDurableProjectionPublication>
                    }),
                },
                definition,
            })))
        }
    }
}
