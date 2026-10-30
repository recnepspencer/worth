//! A genuine model incarnation and its actual independently owned head slot.
use super::super::{
    authentication::Caller, creation, failure::Failure, host::Host, model_query, schema::data::*,
};
use super::branch::{Closed, ForkFailure, InitializationFailure, OwnedBranch};
use worth_query_host::facade::{application_entry::*, product::*};

#[derive(Debug)]
pub struct Model<'host> {
    pub(super) owner: OwnedBranch<'host>,
    pub identity: ModelReference,
    pub target: TargetReference,
}
#[derive(Debug)]
pub enum CreationFailure<'host> {
    BranchCreation(ForkFailure<'host>),
    AfterFork(InitializationFailure<'host>),
}
impl<'host> Model<'host> {
    pub fn create(
        host: &'host Host,
        caller: &Caller<'_>,
        label: &str,
        key: u64,
        days: u64,
    ) -> Result<Self, CreationFailure<'host>> {
        let owner = OwnedBranch::fork(host, host.current_world(), caller)
            .map_err(CreationFailure::BranchCreation)?;
        match initialize(&owner, caller, label, key, days) {
            Ok((identity, target)) => Ok(Self {
                owner,
                identity,
                target,
            }),
            Err(cause) => Err(CreationFailure::AfterFork(InitializationFailure {
                owner,
                cause,
            })),
        }
    }
    pub fn branch(&self) -> WorthQueryProductBranch {
        self.owner.branch()
    }

    pub fn describe(
        &self,
        caller: &Caller<'_>,
    ) -> Result<WorthQueryExpectedProductBasisDescription, Failure> {
        self.owner
            .host
            .with_application_advancement(caller.scope, |phase| {
                self.owner
                    .host
                    .on_branch(self.branch())
                    .select(&phase)
                    .map(|selected| selected.describe_basis())
            })
            .map_err(Failure::Advancement)?
            .map_err(Failure::Selection)
    }
    pub fn read(&self, caller: &Caller<'_>) -> Result<model_query::TargetRow, Failure> {
        read_target(self.owner.host, self.branch(), caller, &self.identity.label)
    }
    pub fn close(self, caller: &Caller<'_>) -> Result<Closed, super::branch::CloseFailure<'host>> {
        self.owner.close(caller)
    }
}

fn initialize(
    owner: &OwnedBranch<'_>,
    caller: &Caller<'_>,
    label: &str,
    key: u64,
    days: u64,
) -> Result<(ModelReference, TargetReference), Failure> {
    let outcome = owner
        .host
        .request(&caller.principal, caller.scope)
        .on_branch(owner.branch())
        .advanced()
        .mutate(creation::CreateModel {
            name: label.to_owned(),
            key,
            days,
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            owner.host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .map_err(Failure::Mutation)?;
    match outcome {
        WorthQueryApplicationMutationOutcome::Committed { .. } => {}
        other => return Err(Failure::ModelCreation(other)),
    }
    let model = owner
        .host
        .request(&caller.principal, caller.scope)
        .on_branch(owner.branch())
        .query(model_query::ReadModel {
            model: label.to_owned(),
        })
        .execute()
        .map_err(Failure::Query)?;
    let model = model.rows().first().cloned().ok_or(Failure::MissingModel)?;
    let target = read_target(owner.host, owner.branch(), caller, label)?;
    let runtime = owner
        .host
        .with_application_advancement(caller.scope, |phase| {
            owner
                .host
                .on_branch(owner.branch())
                .select(&phase)
                .map(|selected| {
                    selected
                        .product()
                        .relational_basis_descriptor()
                        .runtime_instance_id()
                })
        })
        .map_err(Failure::Advancement)?
        .map_err(Failure::Selection)?;
    Ok((
        ModelReference {
            label: model.name,
            runtime,
            occurrence: model.occurrence,
        },
        TargetReference {
            label: target.name,
            occurrence: target.occurrence,
        },
    ))
}

fn read_target(
    host: &Host,
    branch: WorthQueryProductBranch,
    caller: &Caller<'_>,
    label: &str,
) -> Result<model_query::TargetRow, Failure> {
    let rows = host
        .request(&caller.principal, caller.scope)
        .on_branch(branch)
        .query(model_query::ReadTarget {
            model: label.to_owned(),
        })
        .execute()
        .map_err(Failure::Query)?;
    rows.rows().first().cloned().ok_or(Failure::MissingTarget)
}
