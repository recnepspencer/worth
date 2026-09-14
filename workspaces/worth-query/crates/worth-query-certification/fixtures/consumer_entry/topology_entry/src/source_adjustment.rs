use std::marker::PhantomData;

use worth_query_consumer_values::{
    PlanarAdjustmentResult, PlanarMutationDenial, PositiveLength,
};
use worth_query_decl::facade::{
    application_operation::*, application_schema::*, worth_query_operation,
    worth_query_operation_reads, worth_query_operation_writes, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryInvariantMutationTarget,
};

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanarSourceAdjustment {
    pub scope_key: String,
    pub replacement_y: PositiveLength,
}

worth_query_structured_value_binding!(pub PlanarSourceAdjustmentInputBinding for PlanarSourceAdjustment {
    identity: "worth.query.certification.planar-source-adjustment-input.v1"
});
worth_query_structured_value_binding!(pub PlanarSourceAdjustmentResultBinding for PlanarAdjustmentResult {
    identity: "worth.query.certification.planar-source-adjustment-result.v1"
});
worth_query_structured_value_binding!(pub PlanarSourceAdjustmentDenialBinding for PlanarMutationDenial {
    identity: "worth.query.certification.planar-source-adjustment-denial.v1"
});
worth_query_operation!(pub AdjustPlanarSource for Schema: TopologySchemaBinding, input PlanarSourceAdjustmentInputBinding);
worth_query_operation_reads!(AdjustPlanarSource => [Body, BodyKey, PositionY]);
worth_query_operation_writes!(AdjustPlanarSource => [PositionY]);

pub struct PlanarSourceAdjustmentBinding<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> ApplicationMutationBinding<Schema>
    for PlanarSourceAdjustmentBinding<Schema>
{
    type Input = PlanarSourceAdjustment;
    type InputBinding = PlanarSourceAdjustmentInputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarSourceAdjustmentResultBinding;
    type IdempotencyKey = u64;
    type Operation = AdjustPlanarSource;
    type Decision = WorthQueryInvariantMutationTarget<Schema, Body>;
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarSourceAdjustmentDenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;

    const IDENTITY: &'static str = "worth.query.certification.planar-source-adjustment.v1";
    const HANDLER_IDENTITY: &'static str =
        "worth.query.certification.planar-source-adjustment-handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str =
        "worth.query.certification.planar-source-adjustment-command.v1";
    const CANDIDATES: ApplicationCandidateRequirements = requirements(0, 0, 0, 1, 1024, 4096);

    fn idempotency_key_identity(key: &u64) -> [u8; 32] {
        super::mutation_identity::key_identity(*key)
    }

    fn input_identity(input: &PlanarSourceAdjustment) -> [u8; 32] {
        let mutation = PlanarMutation {
            scope_key: input.scope_key.clone(),
            operation: worth_query_consumer_values::PlanarOperation::Adjust(vec![
                worth_query_consumer_values::PlanarAdjustment {
                    body_key: input.scope_key.clone(),
                    replacement_y: input.replacement_y,
                },
            ]),
            validator_work: 0,
        };
        super::mutation_identity::input_identity(&mutation)
    }

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

impl<Schema: TopologySchemaBinding> ApplicationMutationIntent<Schema> for PlanarSourceAdjustment {
    type Binding = PlanarSourceAdjustmentBinding<Schema>;

    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

pub struct PlanarSourceAdjustmentHandler;

impl<Schema: TopologySchemaBinding> OperationHandler<Schema, PlanarSourceAdjustmentBinding<Schema>>
    for PlanarSourceAdjustmentHandler
{
    fn decide(
        &self,
        input: &PlanarSourceAdjustment,
        reader: &mut DecisionReader<'_, '_, '_, Schema, PlanarSourceAdjustmentBinding<Schema>>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Body>, PlanarMutationDenial> {
        let entity = match reader.resolve_entity(BodyKey::reference(), input.scope_key.clone()) {
            Ok(entity) => entity,
            Err(error) => return HandlerResult::ExecutionDenied(error),
        };
        match reader.field(&entity, PositionY::reference()) {
            Ok(Some(_)) => {}
            Ok(None) => {
                return HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate)
            }
            Err(error) => return HandlerResult::ExecutionDenied(error),
        }
        match reader.mutation_target(&entity) {
            Ok(target) => HandlerResult::Completed(target),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }

    fn candidate_requirements(
        &self,
        _: &PlanarSourceAdjustment,
        _: &WorthQueryInvariantMutationTarget<Schema, Body>,
    ) -> ApplicationCandidateRequirements {
        requirements(0, 0, 0, 1, 1024, 4096)
    }

    fn build_candidate(
        &self,
        input: &PlanarSourceAdjustment,
        target: WorthQueryInvariantMutationTarget<Schema, Body>,
        writer: &mut CandidateWriter<'_, Schema, PlanarSourceAdjustmentBinding<Schema>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let entity = match writer.projected_entity(&target) {
            Ok(entity) => entity,
            Err(error) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
            }
        };
        match writer.write_field(&entity, PositionY::reference(), input.replacement_y) {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 1,
            }),
            Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
        }
    }
}

pub(crate) fn declare_planar_source_adjustment<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = AdjustPlanarSource::reference::<Schema>();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 16)
        .operation_projection_work_budget(operation, 16)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, PositionY::reference())
        .operation_write(operation, PositionY::reference())
        .application_mutation_binding::<PlanarSourceAdjustmentBinding<Schema>>()
}
