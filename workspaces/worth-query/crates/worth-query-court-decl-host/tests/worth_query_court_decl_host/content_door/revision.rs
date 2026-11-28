//! Recorded inverse restores genuine Native content custody, not rendered data.
use super::*;
use worth_query_decl::facade::{application_aftermath::*, application_query::*};
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Revise {
    pub key: String,
    pub samples: Vec<i64>,
    pub reference: Vec<u8>,
}
worth_query_structured_value_binding!(pub Input for Revise { identity: "content-revision.input.v1" });
#[derive(Debug)]
pub struct Revised;
worth_query_structured_value_binding!(pub ResultBinding for Revised { identity: "content-revision.result.v1" });
worth_query_operation!(pub ReviseSamples for Schema, input Input; access { read_fields: [Key::at("Key"), Values::at("Values")]; writes: [Values::at("Values")]; });
fn key(input: &Revise) -> String {
    input.key.clone()
}
worth_query_mutation_binding!(pub Binding for Revise, schema Schema,
 identity "content-revision.binding.v1", input Input, operation ReviseSamples, result ResultBinding,
 idempotency String, identity "content-revision.idempotency.v1", decision WorthQueryInvariantMutationTarget<Schema, Record>,
 denial operation::DenialBinding, handler identity "content-revision.handler.v1", program required, outputs NoApplicationMutationOutputs,
 principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
 scope Key, field Key::at("Key"), value key,
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema.application_mutation_binding::<Binding>()
}
pub struct Handler;
impl OperationHandler<Schema, Binding> for Handler {
    fn decide(
        &self,
        input: &Revise,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Record>, operation::Denial> {
        let result = (|| {
            let row = reader.resolve_entity(Key::at("Key"), input.key.clone())?;
            reader
                .content(&row, samples())
                .map_err(HandlerExecutionDenial::new)?;
            reader.mutation_target(&row)
        })();
        match result {
            Ok(row) => HandlerResult::Completed(row),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Revise,
        target: WorthQueryInvariantMutationTarget<Schema, Record>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<Revised, operation::Denial> {
        let result = (|| {
            let row = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            if input.reference.is_empty() {
                let grid = codec::Grid {
                    rows: 1,
                    columns: input.samples.len() as u32,
                    samples: input.samples.clone(),
                    corrupt_result: false,
                    swallow_sink_refusal: false,
                    cancel_during_encoding: None,
                };
                writer
                    .write_content(&row, samples(), &grid)
                    .map_err(HandlerExecutionDenial::new)
            } else {
                let id = ContentRefId(
                    input
                        .reference
                        .as_slice()
                        .try_into()
                        .expect("correction carries a full identity"),
                );
                writer
                    .write_field(&row, Values::at("Values"), BoundReference::new(id))
                    .map_err(HandlerExecutionDenial::new)
            }
        })();
        match result {
            Ok(()) => HandlerResult::Completed(Revised),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
pub struct RegisterRevision;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for RegisterRevision {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                ReviseSamples::reference().definition().no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<Binding>())
    }
}
