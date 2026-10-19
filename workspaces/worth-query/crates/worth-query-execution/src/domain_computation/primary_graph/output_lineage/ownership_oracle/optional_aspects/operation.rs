use super::*;
use fixture_values::{PlanarAdjustmentResult, PlanarMutationDenial};
use worth_query_decl::facade::worth_query_structured_value_binding;
macro_rules! input {
    ($input:ident, $binding:ident, $keep:literal, $identity:literal) => {
        #[derive(Clone, Debug, serde::Serialize)]
        pub(super) struct $input { pub scope_key: String, pub incomplete: bool }
        worth_query_structured_value_binding!(pub(super) $binding for $input { identity: $identity });
        impl<Schema: BirthSchema> ApplicationMutationIntent<Schema> for $input {
            type Binding = Binding<Schema, $keep>;
            fn input(&self) -> &Self { self }
            fn scope_binding(&self) -> PlanarMutationScope<Schema> { PlanarMutationScope::new(BodyKey::reference(), self.scope_key.clone()) }
        }
    };
}
input!(
    Input,
    InputBinding,
    false,
    "worth.query.test.optional-birth-input.v1"
);
input!(
    KeepInput,
    KeepInputBinding,
    true,
    "worth.query.test.optional-keep-input.v1"
);
worth_query_operation!(pub(super) Produce for Schema: BirthSchema, input InputBinding);
worth_query_operation_reads!(Produce => [Body, BodyKey, SourceContent, SourceSelection, OutputNode, PresentNode, FieldlessNode, StrictNode, Incidence]);
worth_query_operation_writes!(Produce => [Value, Extra, SourceContent, SourceSelection]);
worth_query_operation_creates!(Produce => [OutputNode, PresentNode, FieldlessNode, StrictNode]);
worth_query_operation_links!(Produce => [Incidence]);
worth_query_operation!(pub(super) OrdinaryProduce for Schema: BirthSchema, input InputBinding);
worth_query_operation_reads!(OrdinaryProduce => [Body, BodyKey, SourceContent, SourceSelection, OutputNode, PresentNode, FieldlessNode, StrictNode, Incidence]);
worth_query_operation_writes!(OrdinaryProduce => [Value, Extra]);
worth_query_operation_creates!(OrdinaryProduce => [OutputNode, PresentNode, FieldlessNode, StrictNode]);
worth_query_operation_links!(OrdinaryProduce => [Incidence]);
worth_query_operation!(pub(super) KeepProduce for Schema: BirthSchema, input KeepInputBinding);
worth_query_operation_reads!(KeepProduce => [Note, Body, BodyKey, SourceContent, SourceSelection, OutputNode, PresentNode, FieldlessNode, StrictNode, Incidence]);
worth_query_operation_writes!(KeepProduce => [Value, Extra]);
worth_query_operation_creates!(KeepProduce => [OutputNode, PresentNode, FieldlessNode, StrictNode]);
worth_query_operation_links!(KeepProduce => [Incidence]);
pub(super) struct OrdinaryBinding<Schema>(PhantomData<fn() -> Schema>);
pub(super) struct Binding<Schema, const KEEP: bool>(PhantomData<fn() -> Schema>);
pub(super) struct Decision<Schema> {
    source: Option<WorthQueryInvariantMutationTarget<Schema, Body>>,
    node: Option<WorthQueryInvariantMutationTarget<Schema, OutputNode>>,
    present: Option<WorthQueryInvariantMutationTarget<Schema, PresentNode>>,
    fieldless: Option<WorthQueryInvariantMutationTarget<Schema, FieldlessNode>>,
    strict: Option<WorthQueryInvariantMutationTarget<Schema, StrictNode>>,
}
macro_rules! binding {
    ($binding:ty, $keep:literal, $input:ty, $input_binding:ty, $operation:ty, $outputs:ty, $identity:expr) => {
        impl<Schema: BirthSchema> ApplicationMutationBinding<Schema> for $binding {
            type Input = $input;
            type InputBinding = $input_binding;
            type Result = PlanarAdjustmentResult;
            type ResultBinding = PlanarMutationResultBinding;
            type IdempotencyKey = u64;
            type Operation = $operation;
            type Decision = Decision<Schema>;
            type Denial = PlanarMutationDenial;
            type DenialBinding = PlanarMutationDenialBinding;
            type Output = $outputs;
            type ScopeBinding = PlanarMutationScope<Schema>;
            type PrincipalBinding = ConsumerPrincipalBinding;
            type Mapping = ExternalPrincipalMapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            type SourceExpectation = ApplicationQueryMutationSource<PlanarQuery>;
            const IDENTITY: &'static str = $identity;
            const HANDLER_IDENTITY: &'static str = Self::IDENTITY;
            const IDEMPOTENCY_IDENTITY: &'static str = Self::IDENTITY;
            const CANDIDATES: ApplicationCandidateContract =
                ApplicationCandidateContract::FixedShape(requirements(
                    4,
                    1,
                    0,
                    2 + if $keep {
                        0
                    } else {
                        Schema::SOURCE_WRITE.fields()
                    },
                    8192,
                ));
            fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
                ApplicationWorkflowExecutionRequirements::not_applicable()
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
    };
}
binding!(Binding<Schema, false>, false, Input, InputBinding, Produce, BirthOutputs, "worth.query.test.optional-birth.v1");
binding!(
    OrdinaryBinding<Schema>,
    false,
    Input,
    InputBinding,
    OrdinaryProduce,
    BirthOutputs,
    "worth.query.test.optional-ordinary-birth.v1"
);
binding!(Binding<Schema, true>, true, KeepInput, KeepInputBinding, KeepProduce, KeepOutputs, "worth.query.test.optional-keep.v1");
pub(super) struct Handler;
mod birth;
impl<Schema: BirthSchema> OperationHandler<Schema, Binding<Schema, true>> for Handler {
    fn decide(
        &self,
        input: &KeepInput,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding<Schema, true>>,
    ) -> HandlerResult<Decision<Schema>, PlanarMutationDenial> {
        let result = (|| {
            super::tracked_reads::preserve(reader, &input.scope_key)?;
            let node = reader.prior_output::<Binding<Schema, false>, BornNode<Schema>>()?;
            let _note = reader.field(&node, Note::reference())?;
            let present = reader.prior_output::<Binding<Schema, false>, BornPresent<Schema>>()?;
            let fieldless =
                reader.prior_output::<Binding<Schema, false>, BornFieldless<Schema>>()?;
            let strict = reader.prior_output::<Binding<Schema, false>, BornStrict<Schema>>()?;
            let incidence = reader
                .relations_from(Incidence::reference(), &node)
                .map_err(HandlerExecutionDenial::new)?;
            assert_eq!(
                incidence.len(),
                1,
                "the actual birth incidence remains complete before Preserve"
            );
            assert!(
                incidence[0].to() == &fieldless,
                "incidence retains its actual endpoint"
            );
            Ok::<_, HandlerExecutionDenial>(Decision {
                source: None,
                node: Some(reader.mutation_target(&node)?),
                present: Some(reader.mutation_target(&present)?),
                fieldless: Some(reader.mutation_target(&fieldless)?),
                strict: Some(reader.mutation_target(&strict)?),
            })
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(stop) => HandlerResult::ExecutionDenied(stop),
        }
    }
    fn candidate_requirements(
        &self,
        _: &KeepInput,
        _: &Decision<Schema>,
    ) -> ApplicationCandidateRequirements {
        Binding::<Schema, true>::CANDIDATES
            .fixed_requirements()
            .unwrap()
    }
    fn build_candidate(
        &self,
        _: &KeepInput,
        decision: Decision<Schema>,
        writer: &mut CandidateWriter<'_, Schema, Binding<Schema, true>>,
    ) -> HandlerResult<PlanarAdjustmentResult, PlanarMutationDenial> {
        let result = (|| {
            writer
                .preserve_output::<KeptNode<Schema>>(
                    &writer
                        .projected_entity(decision.node.as_ref().unwrap())
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .preserve_output::<KeptPresent<Schema>>(
                    &writer
                        .projected_entity(decision.present.as_ref().unwrap())
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .preserve_output::<KeptFieldless<Schema>>(
                    &writer
                        .projected_entity(decision.fieldless.as_ref().unwrap())
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .preserve_output::<KeptStrict<Schema>>(
                    &writer
                        .projected_entity(decision.strict.as_ref().unwrap())
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            Ok::<_, HandlerExecutionDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(PlanarAdjustmentResult {
                changed_vertices: 0,
            }),
            Err(stop) => HandlerResult::ExecutionDenied(stop),
        }
    }
}
pub(super) fn declare<Schema: BirthSchema>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    macro_rules! declare_operation {
        ($schema:expr, $operation:ident, $binding:ty) => {{
            let schema = $schema;
            let op = $operation::reference::<Schema>();
            schema
                .operation(op.definition().no_external_effect().no_aftermath().finish())
                .operation_projection_work_budget(op, 64)
                .operation_read_entity(op, Body::reference())
                .operation_read_field(op, BodyKey::reference())
                .operation_read_field(op, SourceContent::reference())
                .operation_read_field(op, SourceSelection::reference())
                .operation_read_entity(op, OutputNode::reference())
                .operation_read_entity(op, PresentNode::reference())
                .operation_read_entity(op, FieldlessNode::reference())
                .operation_read_entity(op, StrictNode::reference())
                .operation_read_relation(op, Incidence::reference())
                .operation_create(op, OutputNode::reference())
                .operation_create(op, PresentNode::reference())
                .operation_create(op, FieldlessNode::reference())
                .operation_create(op, StrictNode::reference())
                .operation_write(op, Value::reference())
                .operation_write(op, Extra::reference())
                .operation_link(op, Incidence::reference())
                .application_mutation_binding::<$binding>()
        }};
    }
    declare_operation!(
        declare_operation!(declare_operation!(schema, OrdinaryProduce, OrdinaryBinding<Schema>), Produce, Binding<Schema, false>),
        KeepProduce,
        Binding<Schema, true>
    )
    .operation_read_field(KeepProduce::reference(), Note::reference())
    .operation_write(Produce::reference(), SourceContent::reference())
    .operation_write(Produce::reference(), SourceSelection::reference())
}
