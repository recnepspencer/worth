//! An isolated branch persists typed proposal data through ordinary requests.
use super::super::{
    authentication::Caller, failure::Failure, recording, schema::data::*, storage_query,
};
use super::{
    branch::{ForkFailure, OwnedBranch},
    model::Model,
};
use worth_query_host::facade::{application_entry::*, product::*};

#[derive(Debug)]
pub struct ProposalReference {
    pub key: String,
    pub occurrence: Coordinates,
    pub model: ModelReference,
    pub revision: u64,
}
pub struct Storage<'host> {
    owner: OwnedBranch<'host>,
    model: ModelReference,
}
#[derive(Debug)]
pub enum RecordFailure {
    Request(Failure),
    CommittedObservation {
        key: String,
        idempotency: u64,
        original: StoredProposal,
        outcome: WorthQueryApplicationMutationOutcome<recording::Denial, recording::Recorded>,
        observation: Failure,
    },
}
impl From<Failure> for RecordFailure {
    fn from(cause: Failure) -> Self {
        Self::Request(cause)
    }
}
impl<'host> Storage<'host> {
    pub fn create(model: &Model<'host>, caller: &Caller<'_>) -> Result<Self, ForkFailure<'host>> {
        let owner = OwnedBranch::fork(model.owner.host, model.branch(), caller)?;
        Ok(Self {
            owner,
            model: model.identity.clone(),
        })
    }
    pub fn branch(&self) -> WorthQueryProductBranch {
        self.owner.branch()
    }

    pub fn record(
        &self,
        model: &Model<'host>,
        caller: &Caller<'_>,
        key: &str,
        idempotency: u64,
        intents: Vec<AuthoredIntent>,
    ) -> Result<ProposalReference, RecordFailure> {
        if model.identity != self.model {
            return Err(Failure::ModelMismatch.into());
        }
        let basis = model.describe(caller)?;
        let data = StoredProposal {
            format_version: 1,
            revision: 1,
            model: model.identity.clone(),
            expected_basis: serde_json::to_string(&basis).map_err(Failure::Encoding)?,
            intents,
        };
        let outcome = self
            .owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch())
            .advanced()
            .mutate(recording::RecordProposal {
                model: self.model.label.clone(),
                key: key.to_owned(),
                data: data.clone(),
            })
            .without_source()
            .idempotency(&idempotency)
            .execute_in_program(
                self.owner.host,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(Failure::Mutation)?;
        let committed = matches!(
            &outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
                | WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
                | WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_)
        );
        if !committed {
            return Err(Failure::Recording(outcome).into());
        }
        let row = self.read_key(caller, key).map_err(|observation| {
            RecordFailure::CommittedObservation {
                key: key.to_owned(),
                idempotency,
                original: data,
                outcome,
                observation,
            }
        })?;
        Ok(ProposalReference {
            key: row.key,
            occurrence: row.occurrence,
            model: row.data.model,
            revision: row.data.revision,
        })
    }

    pub fn load(
        &self,
        caller: &Caller<'_>,
        reference: &ProposalReference,
    ) -> Result<StoredProposal, Failure> {
        if self.model != reference.model {
            return Err(Failure::ModelMismatch);
        }
        let row = self.read_key(caller, &reference.key)?;
        if row.occurrence != reference.occurrence {
            return Err(Failure::ProposalOccurrenceChanged);
        }
        if row.data.revision != reference.revision {
            return Err(Failure::ProposalRevisionChanged);
        }
        Ok(row.data)
    }
    fn read_key(
        &self,
        caller: &Caller<'_>,
        key: &str,
    ) -> Result<storage_query::ProposalRow, Failure> {
        let rows = self
            .owner
            .host
            .request(&caller.principal, caller.scope)
            .on_branch(self.branch())
            .query(storage_query::ReadProposals {
                model: self.model.label.clone(),
            })
            .execute()
            .map_err(Failure::Query)?;
        // One-shot execution refuses an oversized collection rather than paging it.
        // Select only after that complete ordinary read (and its disclosure admission).
        let mut matching = rows.rows().iter().filter(|row| row.key == key);
        match (matching.next(), matching.next()) {
            (Some(row), None) => Ok(row.clone()),
            (None, _) => Err(Failure::ProposalUnavailable),
            (Some(_), Some(_)) => Err(Failure::AmbiguousProposal),
        }
    }
    pub fn close(
        self,
        caller: &Caller<'_>,
    ) -> Result<super::branch::Closed, super::branch::CloseFailure<'host>> {
        self.owner.close(caller)
    }
}
