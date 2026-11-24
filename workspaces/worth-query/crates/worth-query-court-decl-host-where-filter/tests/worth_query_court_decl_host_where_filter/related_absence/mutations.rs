//! Mutations for toggling task status, editing notes, linking/unlinking
//! `BlockedBy` edges, and observing a retained `ReadyTasks` source footprint.
use super::queries::ReadyTasksQuery;
use super::schema::*;
use worth_query_decl::facade::{
    application_operation::*, application_query::*, application_schema::*, *,
};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, WorthValue)]
pub struct MutateBoard {
    pub project: u64,
    pub op: u8,
    pub task: u64,
    pub other: u64,
    pub text: String,
}
#[derive(Clone, Debug)]
pub struct Mutated;
#[derive(Clone, Debug)]
pub enum Denied {
    MissingEntity,
}
worth_query_structured_value_binding!(pub MutateInput for MutateBoard { identity: "related-absence.mutate.input.v1" });
worth_query_structured_value_binding!(pub MutateResult for Mutated { identity: "related-absence.mutate.result.v1" });
worth_query_structured_value_binding!(pub MutateDenial for Denied { identity: "related-absence.mutate.denial.v1" });
worth_query_operation!(pub MutateBoardOp for Schema, input MutateInput);
worth_query_operation_reads!(MutateBoardOp => [Contains, BlockedBy]);
worth_query_operation_links!(MutateBoardOp => [BlockedBy]);
worth_query_operation_unlinks!(MutateBoardOp => [BlockedBy]);

pub enum Plan {
    SetStatus(WorthQueryInvariantMutationTarget<Schema, Task>, String),
    SetNotes(WorthQueryInvariantMutationTarget<Schema, Task>, String),
    Link(
        WorthQueryInvariantMutationTarget<Schema, Task>,
        WorthQueryInvariantMutationTarget<Schema, Task>,
    ),
    Unlink(
        WorthQueryInvariantMutationTarget<Schema, Task>,
        WorthQueryInvariantMutationTarget<Schema, Task>,
    ),
    Noop,
}

fn mutate_project(input: &MutateBoard) -> u64 {
    input.project
}
worth_query_mutation_binding!(pub MutateBinding for MutateBoard, schema Schema,
    identity "related-absence.mutate.binding.v1", input MutateInput, operation MutateBoardOp,
    result MutateResult, idempotency u64, identity "related-absence.mutate.key.v1",
    decision Plan, denial MutateDenial,
    handler identity "related-absence.mutate.handler.v1",
    admission ApplicationConventionalAdmission, program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope ProjectKey, field ProjectKey::at("ProjectKey"), value mutate_project,
);

pub struct MutateHandler;
impl OperationHandler<Schema, MutateBinding> for MutateHandler {
    fn decide(
        &self,
        input: &MutateBoard,
        reader: &mut DecisionReader<'_, '_, '_, Schema, MutateBinding>,
    ) -> HandlerResult<Plan, Denied> {
        let project = reader.scope().clone();
        let plan = (|| -> Result<Option<Plan>, HandlerExecutionDenial> {
            reader.field(&project, ProjectKey::at("ProjectKey"))?;
            let Some(task) = reader.resolve_optional_entity(TaskKey::at("TaskKey"), input.task)?
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
                    reader.field(&task, Notes::at("Notes"))?;
                    Ok(Some(Plan::SetNotes(
                        reader.mutation_target(&task)?,
                        input.text.clone(),
                    )))
                }
                2 => {
                    let Some(other) =
                        reader.resolve_optional_entity(TaskKey::at("TaskKey"), input.other)?
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
                    let Some(other) =
                        reader.resolve_optional_entity(TaskKey::at("TaskKey"), input.other)?
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
                _ => Ok(Some(Plan::Noop)),
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
        input: &MutateBoard,
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
            Plan::SetNotes(target, notes) => writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)
                .and_then(|task| {
                    writer
                        .write_field(&task, Notes::at("Notes"), notes)
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
            Plan::Noop => Ok(()),
        };
        match effect {
            Ok(()) => HandlerResult::Completed(Mutated),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

#[derive(Clone, Debug, WorthValue)]
pub struct ObserveReady {
    pub project: u64,
}
#[derive(Clone, Debug)]
pub struct Observed;
#[derive(Clone, Debug)]
pub enum ObserveDenied {}
worth_query_structured_value_binding!(pub ObserveInput for ObserveReady { identity: "related-absence.observe.input.v1" });
worth_query_structured_value_binding!(pub ObserveResult for Observed { identity: "related-absence.observe.result.v1" });
worth_query_structured_value_binding!(pub ObserveDenial for ObserveDenied { identity: "related-absence.observe.denial.v1" });
worth_query_operation!(pub ObserveReadyOp for Schema, input ObserveInput;
    access { read_fields: [ProjectKey::at("ProjectKey")]; });

pub struct ObserveBinding;
impl ApplicationMutationAdmissionBinding<Schema> for ObserveBinding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Schema> for ObserveBinding {
    type Input = ObserveReady;
    type InputBinding = ObserveInput;
    type Result = Observed;
    type ResultBinding = ObserveResult;
    type IdempotencyKey = String;
    type Operation = ObserveReadyOp;
    type Decision = ();
    type Denial = ObserveDenied;
    type DenialBinding = ObserveDenial;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<ProjectKey>;
    type PrincipalBinding = PrincipalBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<ReadyTasksQuery>;
    const IDENTITY: &'static str = "related-absence.observe.v1";
    const HANDLER_IDENTITY: &'static str = "related-absence.observe-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "related-absence.observe-key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ProjectKey {
        ProjectKey::at("ProjectKey")
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        PrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        PrincipalBinding::reference()
    }
    fn expected_source_parameters(
        _: &ObserveReady,
    ) -> Result<Option<ApplicationQueryParameterSet<ReadyTasksQuery>>, ApplicationValueEncodeDenial>
    {
        Ok(Some(ApplicationQueryParameterSet::new()))
    }
}
impl ApplicationMutationIntent<Schema> for ObserveReady {
    type Binding = ObserveBinding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(
        &self,
    ) -> <ObserveBinding as ApplicationMutationBinding<Schema>>::ScopeBinding {
        ApplicationMutationFieldScope::new(ProjectKey::at("ProjectKey"), self.project)
    }
}
pub struct ObserveHandler;
impl OperationHandler<Schema, ObserveBinding> for ObserveHandler {
    fn decide(
        &self,
        _: &ObserveReady,
        _: &mut DecisionReader<'_, '_, '_, Schema, ObserveBinding>,
    ) -> HandlerResult<(), ObserveDenied> {
        HandlerResult::Completed(())
    }
    fn build_candidate(
        &self,
        _: &ObserveReady,
        _: (),
        _: &mut CandidateWriter<'_, Schema, ObserveBinding>,
    ) -> HandlerResult<Observed, ObserveDenied> {
        HandlerResult::Completed(Observed)
    }
}

pub struct Registered;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for Registered {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        let op = MutateBoardOp::reference();
        Ok(builder
            .operation(op.definition().no_external_effect().no_aftermath().finish())
            .operation_projection_work_budget(op, 64)
            .operation_read_field(op, ProjectKey::at("ProjectKey"))
            .operation_read_relation(op, Contains::reference())
            .operation_read_relation(op, BlockedBy::reference())
            .operation_read_field(op, TaskKey::at("TaskKey"))
            .operation_read_field(op, Status::at("Status"))
            .operation_read_field(op, Notes::at("Notes"))
            .operation_write(op, Status::at("Status"))
            .operation_write(op, Notes::at("Notes"))
            .operation_link(op, BlockedBy::reference())
            .operation_unlink(op, BlockedBy::reference())
            .application_mutation_binding::<MutateBinding>()
            .declared_operation::<ObserveReadyOp>(
                ObserveReadyOp::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<ObserveBinding>())
    }
}
