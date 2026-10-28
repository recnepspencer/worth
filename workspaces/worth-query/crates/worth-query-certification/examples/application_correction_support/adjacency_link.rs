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
worth_query_operation_reads!(ConfigureAdjacencyLink => [Number, Key, Primary, AuthoredReference]);
worth_query_operation_writes!(ConfigureAdjacencyLink => [Primary]);
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
    scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
    field Key::reference(), value key,
);
impl ApplicationDeclaredOperation<NumericSchema> for ConfigureAdjacencyLink {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
        schema
            .operation_read_entity(Self::reference(), Number::reference())
            .operation_read_field(Self::reference(), Key::reference())
            .operation_read_field(Self::reference(), Primary::reference())
            .operation_read_relation(Self::reference(), AuthoredReference::reference())
            .operation_write(Self::reference(), Primary::reference())
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
    fn decide(
        &self,
        input: &Link,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<Decision, MissingValue> {
        let from = match reader.resolve_entity(Key::reference(), input.key.clone()) {
            Ok(value) => value,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let to = match reader.resolve_entity(Key::reference(), input.other.clone()) {
            Ok(value) => value,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        match reader.field(&from, Primary::reference()) {
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
        if let Err(cause) = writer.write_field(&from, Primary::reference(), input.primary) {
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

pub struct ReverseLink;
impl ApplicationCorrectionBinding<NumericSchema> for ReverseLink {
    const IDENTITY: &'static str = "adjacency-correction.reverse-link.v1";
    type Source = Binding;
    type InverseIntent = Link;
    type Disclosure = query::ReadBinding;
    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        NumericSchema,
        ConfigureAdjacencyLink,
        Link,
        true,
        false,
    > {
        ConfigureAdjacencyLink::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &Link,
    ) -> Result<ApplicationQueryParameterSet<query::ReadQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(query::parameters())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<NumericSchema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(Primary::reference())],
                U64ApplicationValueBinding::encode(&u64::MAX)
                    .unwrap()
                    .semantic_byte_width(),
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &Link,
        material: ApplicationCorrectionSourceMaterial<'_, NumericSchema>,
    ) -> Result<Link, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::{
            InvalidPreImage, MissingPreImage, UnsupportedSourceMeaning,
        };
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let primary = U64ApplicationValueBinding::decode(
            preimage
                .field(&DeclaredPreImageLocus::from_field(Primary::reference()))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        Ok(Link {
            key: original.key.clone(),
            other: original.other.clone(),
            present: !original.present,
            primary,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("reverse-adjacency:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<NumericSchema>>(
        dispatch: ApplicationCorrectionDispatch<NumericSchema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
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
