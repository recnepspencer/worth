//! Mutations for whole-result live subscription courts & 64-seed DST:
//! status toggle, optional `Due` set/clear, `BlockedBy` link/unlink,
//! `Title`/`Notes` edit, `BoardGrant` status revoke/restore, and subscriber
//! principal enable/disable.
use super::queries::date;
use super::schema::*;
use worth_foundational::facade::CanonicalDate;
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, authentication::*, *,
};
use worth_query_host::facade::primary_graph::*;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::operator_identity;

#[derive(Clone, Debug, WorthValue)]
pub struct MutateLiveBoard {
    pub project: String,
    /// 0=Status, 1=Due (due_days<0 => None), 2=Link, 3=Unlink, 4=Title, 5=Notes,
    /// 6=GrantStatus (other=1 active / 2 revoked), 7=PrincipalStatus (other=1 enabled / 0 disabled)
    pub op: u8,
    pub task: u64,
    pub other: u64,
    pub due_days: i64,
    pub text: String,
}
#[derive(Clone, Debug)]
pub struct Mutated;
#[derive(Clone, Debug)]
pub enum Denied {
    MissingEntity,
}
worth_query_structured_value_binding!(pub MutateInput for MutateLiveBoard { identity: "live-whole-result.mutate.input.v1" });
worth_query_structured_value_binding!(pub MutateResult for Mutated { identity: "live-whole-result.mutate.result.v1" });
worth_query_structured_value_binding!(pub MutateDenial for Denied { identity: "live-whole-result.mutate.denial.v1" });
worth_query_operation!(pub MutateLiveBoardOp for Schema, input MutateInput);
worth_query_operation_reads!(MutateLiveBoardOp => [Contains, BlockedBy]);
worth_query_operation_links!(MutateLiveBoardOp => [BlockedBy]);
worth_query_operation_unlinks!(MutateLiveBoardOp => [BlockedBy]);

pub enum Plan {
    SetStatus(WorthQueryInvariantMutationTarget<Schema, Task>, String),
    SetDue(
        WorthQueryInvariantMutationTarget<Schema, Task>,
        Option<CanonicalDate>,
    ),
    Link(
        WorthQueryInvariantMutationTarget<Schema, Task>,
        WorthQueryInvariantMutationTarget<Schema, Task>,
    ),
    Unlink(
        WorthQueryInvariantMutationTarget<Schema, Task>,
        WorthQueryInvariantMutationTarget<Schema, Task>,
    ),
    SetTitle(WorthQueryInvariantMutationTarget<Schema, Task>, String),
    SetNotes(WorthQueryInvariantMutationTarget<Schema, Task>, String),
    SetGrantStatus(WorthQueryInvariantMutationTarget<Schema, BoardGrant>, u64),
    SetPrincipalStatus(
        WorthQueryInvariantMutationTarget<Schema, ExternalMapping>,
        WorthQueryPrincipalMappingStatus,
    ),
    Noop,
}

fn mutate_project(input: &MutateLiveBoard) -> String {
    input.project.clone()
}
worth_query_mutation_binding!(pub MutateBinding for MutateLiveBoard, schema Schema,
    identity "live-whole-result.mutate.binding.v1", input MutateInput, operation MutateLiveBoardOp,
    result MutateResult, idempotency u64, identity "live-whole-result.mutate.key.v1",
    decision Plan, denial MutateDenial,
    handler identity "live-whole-result.mutate.handler.v1",
    admission ApplicationConventionalAdmission, program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ProjectKey, field ProjectKey::at("ProjectKey"), value mutate_project,
);

pub struct MutateHandler;
impl OperationHandler<Schema, MutateBinding> for MutateHandler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[])
    }
    fn decide(
        &self,
        input: &MutateLiveBoard,
        reader: &mut DecisionReader<'_, '_, '_, Schema, MutateBinding>,
    ) -> HandlerResult<Plan, Denied> {
        let project = reader.scope().clone();
        let plan = (|| -> Result<Option<Plan>, HandlerExecutionDenial> {
            reader.field(&project, ProjectKey::at("ProjectKey"))?;
            match input.op {
                6 => {
                    let Some(grant) = reader.resolve_optional_entity(
                        GrantIdentity::at("GrantIdentity"),
                        input.text.clone(),
                    )?
                    else {
                        return Ok(None);
                    };
                    reader.field(&grant, GrantStatus::at("GrantStatus"))?;
                    Ok(Some(Plan::SetGrantStatus(
                        reader.mutation_target(&grant)?,
                        input.other,
                    )))
                }
                7 => {
                    let Some(mapping) = reader.resolve_optional_entity(
                        ExternalIdentityField::at("ExternalIdentityField"),
                        operator_identity(),
                    )?
                    else {
                        return Ok(None);
                    };
                    reader.field(&mapping, MappingStatus::at("MappingStatus"))?;
                    let status = if input.other == 1 {
                        WorthQueryPrincipalMappingStatus::Enabled
                    } else {
                        WorthQueryPrincipalMappingStatus::Disabled
                    };
                    Ok(Some(Plan::SetPrincipalStatus(
                        reader.mutation_target(&mapping)?,
                        status,
                    )))
                }
                _ => {
                    let Some(task) =
                        reader.resolve_optional_entity(TaskKey::at("TaskKey"), input.task)?
                    else {
                        return Ok(None);
                    };
                    match input.op {
                        0 => {
                            reader.field(&task, Status::at("Status"))?;
                            Ok(Some(Plan::SetStatus(
                                reader.mutation_target(&task)?,
                                input.text.clone(),
                            )))
                        }
                        1 => {
                            reader.field(&task, Due::at("Due"))?;
                            let due = (input.due_days >= 0).then(|| date(input.due_days));
                            Ok(Some(Plan::SetDue(reader.mutation_target(&task)?, due)))
                        }
                        2 => {
                            let Some(other) = reader
                                .resolve_optional_entity(TaskKey::at("TaskKey"), input.other)?
                            else {
                                return Ok(None);
                            };
                            let already = reader
                                .relations_from(BlockedBy::reference(), &task)
                                .map_err(HandlerExecutionDenial::new)?
                                .into_iter()
                                .any(|rel| rel.to().entity_id() == other.entity_id());
                            if already {
                                return Ok(Some(Plan::Noop));
                            }
                            Ok(Some(Plan::Link(
                                reader.mutation_target(&task)?,
                                reader.mutation_target(&other)?,
                            )))
                        }
                        3 => {
                            let Some(other) = reader
                                .resolve_optional_entity(TaskKey::at("TaskKey"), input.other)?
                            else {
                                return Ok(None);
                            };
                            let exists = reader
                                .relations_from(BlockedBy::reference(), &task)
                                .map_err(HandlerExecutionDenial::new)?
                                .into_iter()
                                .any(|rel| rel.to().entity_id() == other.entity_id());
                            if exists {
                                return Ok(Some(Plan::Unlink(
                                    reader.mutation_target(&task)?,
                                    reader.mutation_target(&other)?,
                                )));
                            }
                            Ok(Some(Plan::Noop))
                        }
                        4 => {
                            reader.field(&task, Title::at("Title"))?;
                            Ok(Some(Plan::SetTitle(
                                reader.mutation_target(&task)?,
                                input.text.clone(),
                            )))
                        }
                        5 => {
                            reader.field(&task, Notes::at("Notes"))?;
                            Ok(Some(Plan::SetNotes(
                                reader.mutation_target(&task)?,
                                input.text.clone(),
                            )))
                        }
                        _ => Ok(Some(Plan::Noop)),
                    }
                }
            }
        })();
        match plan {
            Ok(Some(plan)) => HandlerResult::Completed(plan),
            Ok(None) => HandlerResult::DomainDenied(Denied::MissingEntity),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }

    fn build_candidate(
        &self,
        input: &MutateLiveBoard,
        plan: Plan,
        writer: &mut CandidateWriter<'_, Schema, MutateBinding>,
    ) -> HandlerResult<Mutated, Denied> {
        let effect = match plan {
            Plan::SetStatus(target, status) => writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)
                .and_then(|task| {
                    writer
                        .write_field(&task, Status::at("Status"), status)
                        .map_err(HandlerExecutionDenial::new)
                }),
            Plan::SetDue(target, due) => writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)
                .and_then(|task| {
                    writer
                        .write_optional_field(&task, Due::at("Due"), due)
                        .map_err(HandlerExecutionDenial::new)
                }),
            Plan::Link(from, to) => writer
                .projected_entity(&from)
                .map_err(HandlerExecutionDenial::new)
                .and_then(|from_h| {
                    let to_h = writer
                        .projected_entity(&to)
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .link(
                            BlockedBy::reference(),
                            format!("blocked-{}-{}", input.task, input.other),
                            &from_h,
                            &to_h,
                        )
                        .map_err(HandlerExecutionDenial::new)
                }),
            Plan::Unlink(from, to) => writer
                .projected_entity(&from)
                .map_err(HandlerExecutionDenial::new)
                .and_then(|from_h| {
                    let to_h = writer
                        .projected_entity(&to)
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .unlink(BlockedBy::reference(), &from_h, &to_h)
                        .map_err(HandlerExecutionDenial::new)
                }),
            Plan::SetTitle(target, title) => writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)
                .and_then(|task| {
                    writer
                        .write_field(&task, Title::at("Title"), title)
                        .map_err(HandlerExecutionDenial::new)
                }),
            Plan::SetNotes(target, notes) => writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)
                .and_then(|task| {
                    writer
                        .write_field(&task, Notes::at("Notes"), notes)
                        .map_err(HandlerExecutionDenial::new)
                }),
            Plan::SetGrantStatus(target, status) => writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)
                .and_then(|grant| {
                    writer
                        .write_field(&grant, GrantStatus::at("GrantStatus"), status)
                        .map_err(HandlerExecutionDenial::new)
                }),
            Plan::SetPrincipalStatus(target, status) => writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)
                .and_then(|mapping| {
                    writer
                        .write_field(&mapping, MappingStatus::at("MappingStatus"), status)
                        .map_err(HandlerExecutionDenial::new)
                }),
            Plan::Noop => Ok(()),
        };
        match effect {
            Ok(()) => HandlerResult::Completed(Mutated),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

pub struct Registered;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for Registered {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        let op = MutateLiveBoardOp::reference();
        Ok(builder
            .operation(op.definition().no_external_effect().no_aftermath().finish())
            .operation_projection_work_budget(op, 64)
            .operation_read_field(op, ProjectKey::at("ProjectKey"))
            .operation_read_relation(op, Contains::reference())
            .operation_read_relation(op, BlockedBy::reference())
            .operation_read_field(op, TaskKey::at("TaskKey"))
            .operation_read_field(op, Title::at("Title"))
            .operation_read_field(op, Notes::at("Notes"))
            .operation_read_field(op, Status::at("Status"))
            .operation_read_field(op, Due::at("Due"))
            .operation_read_field(op, GrantIdentity::at("GrantIdentity"))
            .operation_read_field(op, GrantStatus::at("GrantStatus"))
            .operation_read_field(op, ExternalIdentityField::at("ExternalIdentityField"))
            .operation_read_field(op, MappingStatus::at("MappingStatus"))
            .operation_write(op, Title::at("Title"))
            .operation_write(op, Notes::at("Notes"))
            .operation_write(op, Status::at("Status"))
            .operation_write(op, Due::at("Due"))
            .operation_write(op, GrantStatus::at("GrantStatus"))
            .operation_write(op, MappingStatus::at("MappingStatus"))
            .operation_link(op, BlockedBy::reference())
            .operation_unlink(op, BlockedBy::reference())
            .application_mutation_binding::<MutateBinding>())
    }
}
