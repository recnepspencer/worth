use super::*;
use std::marker::PhantomData;
use worth_query_consumer_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationOutputRole, WorthQueryCurrentOutputRole, WorthQueryCurrentOutputSelection,
    WorthQueryPreserveOutput,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub(super) struct ChainInput {
    pub scope_key: String,
    pub upstream_key: String,
    pub value: PositiveLength,
}
worth_query_structured_value_binding!(pub(super) ChainInputBinding for ChainInput {
    identity: "worth.query.certification.consumed-chain-input.v1"
});
worth_query_operation!(pub(super) PublishChain for Schema: TopologySchemaBinding, input ChainInputBinding);
worth_query_operation_reads!(PublishChain => [Body, BodyKey, Length]);
worth_query_operation_writes!(PublishChain => [Length]);

pub(super) struct ChainBinding<Schema>(PhantomData<fn() -> Schema>);
impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for ChainInput {
    type Binding = ChainBinding<Schema>;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}
impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema> for ChainBinding<Schema> {
    type Input = ChainInput;
    type InputBinding = ChainInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = PublishChain;
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
    type SourceExpectation = ApplicationQueryMutationSource<PlanarOutputQuery>;
    const IDENTITY: &'static str = "worth.query.certification.consumed-chain-operation.v1";
    const HANDLER_IDENTITY: &'static str = "worth.query.certification.consumed-chain-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.consumed-chain-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(0, 0, 0, 1, 8_192, 4_096);
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

pub(super) fn anchor_role<Schema: TopologySchemaBinding>(
) -> WorthQueryApplicationOutputRole<ChainBinding<Schema>, Body, WorthQueryPreserveOutput> {
    WorthQueryApplicationOutputRole::from_static("anchor")
}

pub(super) struct ChainHandler;
impl<Schema: TopologySchemaBinding> OperationHandler<Schema, ChainBinding<Schema>>
    for ChainHandler
{
    fn decide(
        &self,
        input: &ChainInput,
        reader: &mut DecisionReader<'_, '_, '_, Schema, ChainBinding<Schema>>,
    ) -> HandlerResult<(), PlanarMutationDenial> {
        // The target's actual decision read establishes effect selection for
        // the candidate; the upstream read below establishes the consumed edge.
        let target = match reader.resolve_entity(BodyKey::reference(), input.scope_key.clone()) {
            Ok(entity) => entity,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&target, Length::reference()) {
            Ok(Some(value)) if value == input.value => {}
            Ok(_) => return HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate),
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        let upstream = match reader.resolve_entity(BodyKey::reference(), input.upstream_key.clone())
        {
            Ok(entity) => entity,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        let selected = if input.upstream_key == "anchor-a" {
            reader.current_output::<PlanarOutputFamily, Body, Body>(
                &upstream,
                WorthQueryCurrentOutputRole::new("anchor"),
            )
        } else {
            reader.current_output::<ChainFamily, Body, Body>(
                &upstream,
                WorthQueryCurrentOutputRole::new("anchor"),
            )
        };
        let output = match selected {
            Ok(WorthQueryCurrentOutputSelection::Unique(output)) => output,
            Ok(_) => {
                return HandlerResult::DomainDenied(PlanarMutationDenial::CurrentOutputMissing)
            }
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&output, BodyKey::reference()) {
            Ok(Some(key)) if key == input.upstream_key => HandlerResult::Completed(()),
            Ok(_) => HandlerResult::DomainDenied(PlanarMutationDenial::UnexpectedCurrentOutput),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn candidate_requirements(&self, _: &ChainInput, _: &()) -> ApplicationCandidateRequirements {
        ChainBinding::<Schema>::CANDIDATES
    }
    fn build_candidate(
        &self,
        input: &ChainInput,
        _: (),
        writer: &mut CandidateWriter<'_, Schema, ChainBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let result = (|| {
            let entity = writer
                .resolve_entity(BodyKey::reference(), input.scope_key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .preserve_output(anchor_role(), &entity)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&entity, Length::reference(), input.value)
                .map_err(HandlerExecutionDenial::new)?;
            Ok::<_, HandlerExecutionDenial>(PlanarAdjustmentResult {
                changed_vertices: 1,
            })
        })();
        match result {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}

pub(super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = PublishChain::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 32)
        // The actual consumed-output lookup verifies upstream lineage/native
        // currentness inside the handler's projection allowance.
        .operation_projection_work_budget(operation, 4_096)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, Length::reference())
        .operation_write(operation, Length::reference())
        .application_mutation_binding::<ChainBinding<Schema>>()
}
