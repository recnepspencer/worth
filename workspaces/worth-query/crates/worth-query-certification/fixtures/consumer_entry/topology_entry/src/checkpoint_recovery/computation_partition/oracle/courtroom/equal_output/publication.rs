//! Root publication keeps its Native output while its successor input changes.
use super::*;
use worth_query_decl::facade::application_operation::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_decl::facade::{
    worth_query_operation, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::HandlerExecutionDenial;

#[derive(Clone, serde::Serialize)]
pub(super) struct Input {
    pub(super) scope_key: String,
    pub(super) entries: String,
}
worth_query_structured_value_binding!(pub(super) InputBinding for Input { identity: "courtroom-equal-root-input" });
worth_query_operation!(pub(super) PublishRoot for Schema: TopologySchemaBinding, input InputBinding);
worth_query_operation_reads!(PublishRoot => [Body, BodyKey, Length]);
worth_query_operation_writes!(PublishRoot => [Length]);

pub(super) struct Binding;
impl ApplicationMutationBinding<Schema> for Binding {
    type Input = Input;
    type InputBinding = InputBinding;
    type Result = PlanarAdjustmentResult;
    type ResultBinding = PlanarMutationResultBinding;
    type IdempotencyKey = u64;
    type Operation = PublishRoot;
    type Decision = PositiveLength;
    type Denial = PlanarMutationDenial;
    type DenialBinding = PlanarMutationDenialBinding;
    type Output = PlanarOutputs;
    type ScopeBinding = PlanarMutationScope<Schema>;
    type PrincipalBinding = ConsumerPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<source::SuccessorQuery>;
    const IDENTITY: &'static str = "courtroom-equal-root-publication";
    const HANDLER_IDENTITY: &'static str = "courtroom-equal-root-handler";
    const IDEMPOTENCY_IDENTITY: &'static str = "courtroom-equal-root-command";
    const CANDIDATES: ApplicationCandidateRequirements = RegionOutputBinding::<Schema>::CANDIDATES;
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
impl ApplicationMutationIntent<Schema> for Input {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> PlanarMutationScope<Schema> {
        PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone())
    }
}

pub(super) struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Input,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<PositiveLength, PlanarMutationDenial> {
        let value = (|| {
            let entity = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            let _ = reader.field(&entity, Length::reference())?;
            // Quantized output: adjacent positive inputs share a canonical value.
            let value = input
                .entries
                .parse::<u64>()
                .expect("source is a positive length");
            Ok::<_, HandlerExecutionDenial>(Some(length(value.div_ceil(2))))
        })();
        match value {
            Ok(Some(value)) => HandlerResult::Completed(value),
            Ok(None) => HandlerResult::DomainDenied(PlanarMutationDenial::MissingCoordinate),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
    fn candidate_requirements(
        &self,
        _: &Input,
        _: &PositiveLength,
    ) -> ApplicationCandidateRequirements {
        Binding::CANDIDATES
    }
    fn build_candidate(
        &self,
        input: &Input,
        value: PositiveLength,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let result = (|| {
            let entity = writer
                .resolve_entity(BodyKey::reference(), input.scope_key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .preserve_output::<PlanarAnchorOutput<Schema>>(&entity)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&entity, Length::reference(), value)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(PlanarAdjustmentResult {
                changed_vertices: 1,
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}
pub(super) fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let operation = PublishRoot::reference();
    builder
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_decision_fact_budget(operation, 32)
        .operation_projection_work_budget(operation, 4096)
        .operation_read_entity(operation, Body::reference())
        .operation_read_field(operation, BodyKey::reference())
        .operation_read_field(operation, Length::reference())
        .operation_write(operation, Length::reference())
        .application_mutation_binding::<Binding>()
}
