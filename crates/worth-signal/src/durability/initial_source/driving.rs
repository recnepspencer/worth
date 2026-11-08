//! Advance only retained original frames; transfer the same admission on ACK.
use super::*;
use crate::durability::publication_binding::{admit_branch_record, admit_definition};

impl<D, I, E, Ctx, T> SignalInitialSourcePublication<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn definition(&self) -> &SignalDefinitionTransitionArtifact {
        &self.custody.definition
    }
    pub fn advance_artifact(&self) -> &SignalBranchAdvanceArtifact {
        &self.custody.advance
    }

    pub fn advance(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalInitialSourceProgress, SignalInitialSourceStepRefusal> {
        loop {
            match &self.phase {
                InitialSourcePhase::Definition => {
                    let completion = match self
                        .custody
                        .medium
                        .advance_definition(&self.custody.definition, execution)
                    {
                        SignalPublicationObservation::Pending => {
                            return Ok(SignalInitialSourceProgress::Pending)
                        }
                        SignalPublicationObservation::Finished(completion) => completion,
                    };
                    let proof = admit_definition(self.custody.definition.clone(), completion)
                        .map_err(|fate| refusal(SignalDurableJournal::Definitions, fate))?;
                    let custody = self
                        .custody
                        .definition_custody
                        .take()
                        .expect("only the original definition ACK consumes its custody");
                    self.phase = InitialSourcePhase::Advance(
                        SignalDefinitionSourceRecord::from_committed(proof, custody),
                    );
                }
                InitialSourcePhase::Advance(definition) => {
                    let completion = match self
                        .custody
                        .medium
                        .advance_branch(&self.custody.advance, execution)
                    {
                        SignalPublicationObservation::Pending => {
                            return Ok(SignalInitialSourceProgress::Pending)
                        }
                        SignalPublicationObservation::Finished(completion) => completion,
                    };
                    let record = admit_branch_record(
                        self.custody.advance.clone(),
                        Arc::clone(definition),
                        completion,
                    )
                    .map_err(|(_, fate)| refusal(SignalDurableJournal::Advances, fate))?;
                    self.phase = InitialSourcePhase::Published(record);
                }
                InitialSourcePhase::Published(_) => {
                    return Ok(SignalInitialSourceProgress::Published)
                }
            }
        }
    }
    /// A pending original is returned unchanged, never cancelled into no effect.
    pub fn try_into_published(self) -> Result<SignalPublishedInitialSource<D, I, E, Ctx, T>, Self> {
        match self.phase {
            InitialSourcePhase::Published(record) => {
                Ok(SignalPublishedInitialSource::<D, I, E, Ctx, T> {
                    custody: self.custody,
                    record,
                })
            }
            InitialSourcePhase::Definition | InitialSourcePhase::Advance(_) => Err(self),
        }
    }
}
impl<D, I, E, Ctx, T> SignalPublishedInitialSource<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn source_point(&self) -> SignalRecordedSourcePoint {
        self.record.source_point()
    }
    pub fn readmit_authored_owner(
        self,
        installation: &mut dyn SignalAuthoredOwnerInstallation,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<SignalDurableOwner<D, I, E, Ctx, T>, SignalOwnerReadmissionDenial> {
        let InitialSourceCustody {
            program,
            medium,
            contribution,
            vocabulary,
            admission,
            definition: _,
            advance: _,
            definition_custody: _,
            _advance_custody: _,
            _memory,
        } = *self.custody;
        program.readmit_authored_owner_with_admission(
            medium,
            contribution,
            vocabulary,
            installation,
            execution,
            admission,
        )
    }
}
fn refusal(
    journal: SignalDurableJournal,
    fate: DurablePublicationFate<super::super::publication_binding::SignalPublicationBindingDenial>,
) -> SignalInitialSourceStepRefusal {
    let fate = match fate {
        DurablePublicationFate::Committed => DurablePublicationFate::Committed,
        DurablePublicationFate::ProvenNoEffect { detail } => {
            DurablePublicationFate::ProvenNoEffect {
                detail: detail.into(),
            }
        }
        DurablePublicationFate::Indeterminate { detail } => DurablePublicationFate::Indeterminate {
            detail: detail.into(),
        },
    };
    SignalInitialSourceStepRefusal { journal, fate }
}
