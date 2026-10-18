//! A native consumed-output operation whose source includes an ignored sibling value.
use super::super::*;
use std::marker::PhantomData;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial, PositiveLength};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryCurrentOutputSelection,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceInput {
    pub scope_key: String,
    pub value: PositiveLength,
}
worth_query_structured_value_binding!(pub(in crate::checkpoint_recovery::required_chain) ValueSourceInputBinding for ValueSourceInput { identity: "worth.query.certification.consumed-value-source-input.v1" });
worth_query_operation!(pub(in crate::checkpoint_recovery::required_chain) PublishValueSource for Schema: TopologySchemaBinding, input ValueSourceInputBinding);
worth_query_operation_reads!(PublishValueSource => [Body, BodyKey, Length]);
worth_query_operation_writes!(PublishValueSource => [Length]);
pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceBinding<Schema>(
    PhantomData<fn() -> Schema>,
);
impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for ValueSourceInput {
    type Binding = ValueSourceBinding<Schema>;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}
impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for ValueSourceBinding<Schema>
{
    fn workflow_execution_requirements(
    ) -> worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements
    {
        worth_query_decl::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable()
    }

    type Input = ValueSourceInput;
    type InputBinding = ValueSourceInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = PublishValueSource;
    type Decision = ();
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = PlanarOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;
    const IDENTITY: &'static str = "worth.query.certification.consumed-value-source-operation.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.consumed-value-source-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.consumed-value-source-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(0, 0, 0, 1, 8_192);
    fn scope_field() -> ApplicationFieldRef<
        Schema,
        Body,
        PlanarPosition,
        BodyKey,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        BodyKey::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        ConsumerPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ConsumerPrincipalBinding::reference()
    }
}

pub(in crate::checkpoint_recovery::required_chain) struct ValueSourceHandler;
impl<Schema: TopologySchemaBinding> OperationHandler<Schema, ValueSourceBinding<Schema>>
    for ValueSourceHandler
{
    fn decide(
        &self,
        input: &ValueSourceInput,
        reader: &mut DecisionReader<'_, '_, '_, Schema, ValueSourceBinding<Schema>>,
    ) -> HandlerResult<(), PlanarMutationDenial> {
        let read = (|| {
            let target = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            if reader.field(&target, Length::reference())? != Some(input.value) {
                return Ok(false);
            }
            let upstream = reader.resolve_entity(BodyKey::reference(), "anchor-a".to_owned())?;
            let WorthQueryCurrentOutputSelection::Unique(output) =
                reader.current_output::<PlanarOutputFamily, Body>(&upstream)?
            else {
                return Ok(false);
            };
            Ok(reader.field(&output, Length::reference())?.is_some())
        })();
        match read {
            Ok(true) => HandlerResult::Completed(()),
            Ok(false) => HandlerResult::DomainDenied(PlanarMutationDenial::CurrentOutputMissing),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn candidate_requirements(
        &self,
        _: &ValueSourceInput,
        _: &(),
    ) -> ApplicationCandidateRequirements {
        ValueSourceBinding::<Schema>::CANDIDATES
    }
    fn build_candidate(
        &self,
        input: &ValueSourceInput,
        _: (),
        writer: &mut CandidateWriter<'_, Schema, ValueSourceBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let written = (|| {
            let target = writer
                .resolve_entity(BodyKey::reference(), input.scope_key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .preserve_output::<PlanarAnchorOutput<Schema>>(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&target, Length::reference(), input.value)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(PlanarAdjustmentResult {
                changed_vertices: 1,
            })
        })();
        match written {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}
pub(in crate::checkpoint_recovery::required_chain) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = PublishValueSource::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 4096)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, Length::reference())
        .operation_write(operation, Length::reference())
        .application_mutation_binding::<ValueSourceBinding<Schema>>()
}
