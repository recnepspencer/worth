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
                InitialSourcePhase::Definition | InitialSourcePhase::DefinitionRefused(_) => {
                    let observation = match &self.phase {
                        InitialSourcePhase::Definition => self
                            .custody
                            .medium
                            .advance_definition(&self.custody.definition, execution),
                        InitialSourcePhase::DefinitionRefused(proof) => {
                            self.custody.medium.retry_definition_after_no_effect(
                                &self.custody.definition,
                                proof,
                                execution,
                            )
                        }
                        InitialSourcePhase::Advance(_)
                        | InitialSourcePhase::AdvanceRefused { .. }
                        | InitialSourcePhase::Published(_) => unreachable!("definition phase"),
                    };
                    let completion = match observation {
                        SignalPublicationObservation::Pending => {
                            return Ok(SignalInitialSourceProgress::Pending)
                        }
                        SignalPublicationObservation::Finished(completion) => completion,
                    };
                    let proof = match admit_definition(self.custody.definition.clone(), completion)
                    {
                        Ok(proof) => proof,
                        Err(fate) => {
                            self.phase = match &fate {
                                DurablePublicationFate::ProvenNoEffect { .. } => {
                                    InitialSourcePhase::DefinitionRefused(
                                        self.custody.admission.admission().no_effect_original(
                                            SignalDurableJournal::Definitions,
                                            self.custody.definition.digest(),
                                        ),
                                    )
                                }
                                DurablePublicationFate::Committed
                                | DurablePublicationFate::Indeterminate { .. } => {
                                    InitialSourcePhase::Definition
                                }
                            };
                            return Err(refusal(SignalDurableJournal::Definitions, fate));
                        }
                    };
                    let custody = self
                        .custody
                        .definition_custody
                        .take()
                        .expect("only the original definition ACK consumes its custody");
                    self.phase = InitialSourcePhase::Advance(
                        SignalDefinitionSourceRecord::from_committed(proof, custody),
                    );
                }
                InitialSourcePhase::Advance(definition)
                | InitialSourcePhase::AdvanceRefused { definition, .. } => {
                    let definition = Arc::clone(definition);
                    let observation = match &self.phase {
                        InitialSourcePhase::Advance(_) => self
                            .custody
                            .medium
                            .advance_branch(&self.custody.advance, execution),
                        InitialSourcePhase::AdvanceRefused { proof, .. } => self
                            .custody
                            .medium
                            .retry_branch_after_no_effect(&self.custody.advance, proof, execution),
                        InitialSourcePhase::Definition
                        | InitialSourcePhase::DefinitionRefused(_)
                        | InitialSourcePhase::Published(_) => unreachable!("advance phase"),
                    };
                    let completion = match observation {
                        SignalPublicationObservation::Pending => {
                            return Ok(SignalInitialSourceProgress::Pending)
                        }
                        SignalPublicationObservation::Finished(completion) => completion,
                    };
                    let record = admit_branch_record(
                        self.custody.advance.clone(),
                        Arc::clone(&definition),
                        completion,
                    );
                    let record = match record {
                        Ok(record) => record,
                        Err((_, fate)) => {
                            self.phase = match &fate {
                                DurablePublicationFate::ProvenNoEffect { .. } => {
                                    InitialSourcePhase::AdvanceRefused {
                                        definition,
                                        proof: self
                                            .custody
                                            .admission
                                            .admission()
                                            .no_effect_original(
                                                SignalDurableJournal::Advances,
                                                self.custody.advance.digest(),
                                            ),
                                    }
                                }
                                DurablePublicationFate::Committed
                                | DurablePublicationFate::Indeterminate { .. } => {
                                    InitialSourcePhase::Advance(definition)
                                }
                            };
                            return Err(refusal(SignalDurableJournal::Advances, fate));
                        }
                    };
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
            InitialSourcePhase::Definition
            | InitialSourcePhase::DefinitionRefused(_)
            | InitialSourcePhase::Advance(_)
            | InitialSourcePhase::AdvanceRefused { .. } => Err(self),
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
