//! A model configuration edits its authored value and links/unlinks its subject.
use super::{
    authored_identity_model::{AuthoredReference, Target},
    edit::{DenialBinding, Edited, MissingValue, ResultBinding},
    query,
    schema::*,
};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    declaration::{
        application_aftermath::*, application_operation::*, application_program::*,
        application_query::*, application_schema::*,
    },
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Link {
    pub key: String,
    pub other: String,
    pub present: bool,
    pub primary: u64,
}
fn key(input: &Link) -> String {
    input.key.clone()
}
worth_query_structured_value_binding!(pub Input for Link { identity: "adjacency-correction.link.input.v1" });
worth_query_operation!(pub ConfigureAdjacencyLink for NumericSchema, input Input);
worth_query_operation_reads!(ConfigureAdjacencyLink => [Number, AuthoredReference]);

impl OperationLinks<ConfigureAdjacencyLink> for AuthoredReference {}
impl OperationUnlinks<ConfigureAdjacencyLink> for AuthoredReference {}
pub struct Decision {
    from: Target,
    to: Target,
}
worth_query_mutation_binding!(
    pub Binding for Link, schema NumericSchema,
    identity "adjacency-correction.link.v1", input Input, operation ConfigureAdjacencyLink, result ResultBinding,
    idempotency String, identity "adjacency-correction.command.v1",
    decision Decision, denial DenialBinding,
    handler identity "adjacency-correction.link.v1", outputs NoApplicationMutationOutputs,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key,
    field Key::at("Key"), value key,
);
impl ApplicationDeclaredOperation<NumericSchema> for ConfigureAdjacencyLink {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
        schema
            .operation_read_entity(Self::reference(), Number::reference())
            .operation_read_field(Self::reference(), Key::at("Key"))
            .operation_read_field(Self::reference(), Primary::at("Primary"))
            .operation_read_relation(Self::reference(), AuthoredReference::reference())
            .operation_write(Self::reference(), Primary::at("Primary"))
            .operation_link(Self::reference(), AuthoredReference::reference())
            .operation_unlink(Self::reference(), AuthoredReference::reference())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema.application_mutation_binding::<Binding>()
}
pub struct Handler;
impl OperationHandler<NumericSchema, Binding> for Handler {
    fn declared_durable_capabilities() -> Option<
        &'static [worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability],
    > {
        Some(&[worth_query_host::facade::primary_graph::WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &Link,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<Decision, MissingValue> {
        let from = match reader.resolve_entity(Key::at("Key"), input.key.clone()) {
            Ok(value) => value,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let to = match reader.resolve_entity(Key::at("Key"), input.other.clone()) {
            Ok(value) => value,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        match reader.field(&from, Primary::at("Primary")) {
            Ok(Some(_)) => {}
            Ok(None) => return HandlerResult::DomainDenied(MissingValue),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        }
        let relations = match reader
            .reader()
            .decision_relations_from(AuthoredReference::reference(), &from)
        {
            Ok(value) => value,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        let matching = relations
            .iter()
            .filter(|relation| relation.to().entity_id() == to.entity_id())
            .count();
        // The inverse toggles a zero-or-one domain relationship. Parallel
        // membership must not be erased and recreated as one edge.
        if matching != usize::from(!input.present) {
            return HandlerResult::DomainDenied(MissingValue);
        }
        let from = match reader.mutation_target(&from) {
            Ok(value) => value,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        match reader.mutation_target(&to) {
            Ok(to) => HandlerResult::Completed(Decision { from, to }),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Link,
        decision: Decision,
        writer: &mut CandidateWriter<'_, NumericSchema, Binding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let from = match writer.projected_entity(&decision.from) {
            Ok(value) => value,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        let to = match writer.projected_entity(&decision.to) {
            Ok(value) => value,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        if let Err(cause) =
            writer.write_field(&from, Primary::at("Primary"), Counter(input.primary))
        {
            return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
        }
        let result = if input.present {
            writer.link(
                AuthoredReference::reference(),
                format!("adjacency:{}:{}", input.key, input.other),
                &from,
                &to,
            )
        } else {
            writer.unlink(AuthoredReference::reference(), &from, &to)
        };
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}

pub struct RegisterLink;
impl worth_query_decl::facade::application_schema::fragment::Registration<NumericSchema> for RegisterLink {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                ConfigureAdjacencyLink::reference()
            .definition()
            .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<Binding>())
    }
}
pub struct Feature;
impl ApplicationFeature<NumericSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "adjacency-correction.feature.v1";
}
pub struct Completion;
impl ApplicationActionDelivery<NumericSchema, Binding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "adjacency-correction.completion.v1";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<NumericSchema, Feature>()
        .action::<Binding, Completion>()
        .finish_action()
        .finish()
}
