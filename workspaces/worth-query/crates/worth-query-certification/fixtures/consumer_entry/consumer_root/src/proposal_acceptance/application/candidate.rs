//! Internal proposal staging owns the exact candidate; it does not register an acceptance attempt.
use super::super::{
    authentication::Caller,
    failure::Failure,
    host::Host,
    retention,
    schema::{data::*, Schema},
};
use super::{
    branch::{Closed, OwnedBranch},
    model::Model,
    retention_action,
    storage::{ProposalReference, Storage},
};
use worth_query_host::facade::{
    application_contribution::WorthQueryAdvancementDenial, application_entry::*, product::*,
};

pub mod adoption;
pub mod lifecycle;
pub mod preparation;
pub mod staging;
pub use lifecycle::CloseFailure;
pub use preparation::{PreparationFailure, PreparationRefusal};
pub use staging::StageEvent;

/// Descriptive application metadata, prebuilt before any candidate is created.
#[derive(Clone, Debug)]
pub struct AdoptionMetadata {
    pub proposal: ProposalReference,
    pub attempt: u64,
}
pub struct Staging<'host> {
    host: &'host Host,
    candidate: Option<WorthQueryExpectedProductBranch<'host, Schema>>,
    metadata: AdoptionMetadata,
    model_metadata: Option<AdoptionMetadata>,
    proposal: StoredProposal,
    next: usize,
    step: Option<Step>,
    completed: Vec<retention_action::Completion>,
}
enum Step {
    Pending(retention_action::Operation),
    Completed(retention_action::Completed),
    Terminal(retention_action::Terminal),
}
/// Only complete staging can enter the application's guarded head replacement.
pub struct ReadyToAdopt<'host> {
    staging: Staging<'host>,
}
impl Staging<'_> {
    pub fn branch(&self) -> WorthQueryProductBranch {
        self.candidate
            .as_ref()
            .expect("live candidate custody")
            .branch()
    }
    pub fn read(&self, model: &Model<'_>, caller: &Caller<'_>) -> Result<u64, PreparationRefusal> {
        preparation::authorize(model, caller, &self.proposal)?;
        super::model::read_target(self.host, self.branch(), caller, &self.proposal.model.label)
            .map(|row| row.days)
            .map_err(PreparationRefusal::Request)
    }
    pub fn terminal_outcome(
        &self,
        model: &Model<'_>,
        caller: &Caller<'_>,
    ) -> Result<
        Option<&WorthQueryApplicationMutationOutcome<retention::Denial, retention::Changed>>,
        PreparationRefusal,
    > {
        preparation::authorize(model, caller, &self.proposal)?;
        Ok(match &self.step {
            Some(Step::Terminal(terminal)) => terminal.native_outcome(),
            _ => None,
        })
    }
}
impl<'host> Staging<'host> {
    pub fn finish(self) -> Result<ReadyToAdopt<'host>, Self> {
        if self.next == self.proposal.intents.len() && self.step.is_none() {
            Ok(ReadyToAdopt { staging: self })
        } else {
            Err(self)
        }
    }
}
