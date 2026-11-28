//! A declared original output lends content after its live field has changed.
use super::*;
use worth_query_decl::facade::{application_aftermath::*, application_query::*};
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Issue {
    pub key: String,
    pub samples: Vec<i64>,
}
worth_query_structured_value_binding!(pub Input for Issue {identity:"created-content.input.v1"});
worth_query_operation!(pub Create for Schema,input Input);

worth_query_operation_creates!(Create=>[Record]);
worth_query_operation_links!(Create=>[ContainsSamples]);
impl ApplicationDeclaredOperation<Schema> for Create {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        shared_reads::create(schema)
            .operation_write(Self::reference(), Key::at("Key"))
            .operation_write(Self::reference(), Values::at("Values"))
            .operation_create(Self::reference(), Record::reference())
            .operation_link(Self::reference(), ContainsSamples::reference())
    }
}
pub struct Outputs;
pub struct Actual;
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Actual::DESCRIPTOR];
}
impl WorthQueryApplicationOutputRole for Actual {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Record;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "created-samples";
}
fn scope(_: &Issue) -> String {
    "record".into()
}
worth_query_mutation_binding!(pub Binding for Issue,schema Schema,
 identity "created-content.binding.v1",input Input,operation Create,result revision::ResultBinding,
 idempotency String,identity "created-content.key.v1",decision WorthQueryInvariantMutationTarget<Schema,Record>,denial operation::DenialBinding,
 handler identity "created-content.handler.v1",program required,outputs Outputs,
 principal Actor,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
 scope Key, field Key::at("Key"),value scope,
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
        input: &Issue,
        reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Schema, Record>, operation::Denial> {
        let result = reader
            .resolve_entity(Key::at("Key"), "record".to_owned())
            .and_then(|root| {
                if input.key == "undeclared-content-read" {
                    reader
                        .content(&root, samples())
                        .map_err(HandlerExecutionDenial::new)?;
                }
                reader.mutation_target(&root)
            });
        match result {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Issue,
        target: WorthQueryInvariantMutationTarget<Schema, Record>,
        writer: &mut CandidateWriter<'_, Schema, Binding>,
    ) -> HandlerResult<revision::Revised, operation::Denial> {
        let result = (|| {
            let root = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            let child = writer
                .create_entity_in_context(
                    &root,
                    Record::reference(),
                    WorthQueryApplicationEntityKey::new(&input.key).unwrap(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&child, Key::at("Key"), input.key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            let grid = codec::Grid {
                rows: 1,
                columns: input.samples.len() as u32,
                samples: input.samples.clone(),
                corrupt_result: false,
                swallow_sink_refusal: false,
                cancel_during_encoding: None,
            };
            writer
                .initialize_content(&child, samples(), &grid)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    ContainsSamples::reference(),
                    input.key.clone(),
                    &root,
                    &child,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .create_output::<Actual>(&child)
                .map_err(HandlerExecutionDenial::new)?;
            Ok(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(revision::Revised),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

pub struct RegisterCreation;

impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for RegisterCreation {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial> {
        Ok(schema
            .declared_operation(Create::reference().definition().no_external_effect().no_aftermath().finish())
            .application_mutation_binding::<Binding>())
    }
}
