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
        schema
            .operation_read_field(Self::reference(), Key::at("Key"))
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
pub struct Capture<const DAMAGE: u8 = 0>;
fn output() -> ApplicationCorrectionOutputField<Schema> {
    ApplicationCorrectionOutput::<Binding, Actual>::content_field(samples())
}
impl<const DAMAGE: u8> ApplicationCorrectionBinding<Schema> for Capture<DAMAGE> {
    const IDENTITY: &'static str = "created-content.capture.v1";
    type Source = Binding;
    type InverseIntent = revision::Revise;
    type Disclosure = source::query::StateBinding;

    fn recover_source_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    >{
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>>::Input>::new(bounds),
        )
    }

    fn recover_inverse_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<
                Schema,
                Self,
            > as worth_query_decl::facade::application_operation::ApplicationMutationBinding<
                Schema,
            >>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    > {
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<Schema, Self> as worth_query_decl::facade::application_operation::ApplicationMutationBinding<Schema>>::Input>::new(bounds),
        )
    }
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Schema, Create, Issue, true, false> {
        Create::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Issue,
    ) -> Result<
        ApplicationQueryParameterSet<source::query::StateQuery>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::compensation_binding::<Self>(
            DeclaredAftermathPostcondition::BusinessPostcondition {
                identity: "original-content-restored-through-declared-output".into(),
            },
        )
    }
    fn source_output_fields() -> Vec<ApplicationCorrectionOutputField<Schema>> {
        if DAMAGE == 1 {
            vec![
                ApplicationCorrectionOutput::<Binding, Actual>::content_field(content_field(
                    Values::at("MissingValues"),
                    codec::GridFormat,
                )),
            ]
        } else if DAMAGE == 2 {
            vec![
                ApplicationCorrectionOutput::<Binding, Actual>::content_field(content_field(
                    Values::at("Values"),
                    codec::Impostor,
                )),
            ]
        } else {
            vec![output()]
        }
    }
    fn derive_correction(
        _: &Issue,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<revision::Revise, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        let ApplicationCorrectionSourceMaterial::Compensation(fields) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let value = fields
            .field(&output())
            .map_err(|_| UnsupportedSourceMeaning)?;
        let id = ContentRefApplicationValueBinding::decode(value).map_err(|_| InvalidPreImage)?;
        Ok(revision::Revise {
            key: "record".into(),
            samples: vec![],
            reference: id.0.to_vec(),
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("capture:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}

fn refused_content_selection<const DAMAGE: u8>(path: &'static str) {
    let expression = Capture::<DAMAGE>::source_output_fields();
    let field = expression[0].binding_recipe().locus().clone();
    assert_eq!(field.field(), path);
    let denial = ApplicationSchemaDeclarationBuilder::<Schema>::from_fragment(ContentModel)
        .unwrap()
        .correction::<Capture<DAMAGE>>()
        .build()
        .unwrap_err();
    assert_eq!(
        denial,
        ApplicationSchemaDeclarationDenial::InvalidCorrectionField {
            correction: "created-content.capture.v1".into(),
            field: field.clone(),
        }
    );
    let message = denial.to_string();
    for part in [
        "created-content.capture.v1",
        field.entity(),
        field.aspect(),
        field.field(),
    ] {
        assert!(message.contains(part), "{message}");
    }
}
#[test]
fn a_misspelled_content_output_is_refused_at_composition() {
    refused_content_selection::<1>("MissingValues");
}
#[test]
fn a_content_output_codec_mismatch_is_refused_at_composition() {
    refused_content_selection::<2>("Values");
}
