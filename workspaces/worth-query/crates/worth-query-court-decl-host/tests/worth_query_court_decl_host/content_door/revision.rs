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
pub struct Restore;
impl ApplicationCorrectionBinding<Schema> for Restore {
    const IDENTITY: &'static str = "content-revision.restore.v1";
    type Source = Binding;
    type InverseIntent = Revise;
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
    ) -> ApplicationOperationDefinitionBuilder<Schema, ReviseSamples, Revise, true, false> {
        ReviseSamples::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Revise,
    ) -> Result<
        ApplicationQueryParameterSet<source::query::StateQuery>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(DeclaredPreImageDemand::new(
            [DeclaredPreImageLocus::from_field(Values::at("Values"))],
            4096,
        ))
    }
    fn derive_correction(
        original: &Revise,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<Revise, ApplicationCorrectionDerivationDenial> {
        let ApplicationCorrectionSourceMaterial::RecordedInverse(prior) = material else {
            return Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning);
        };
        let value = prior
            .field(&DeclaredPreImageLocus::from_field(Values::at("Values")))
            .ok_or(ApplicationCorrectionDerivationDenial::MissingPreImage)?;
        let id = ContentRefApplicationValueBinding::decode(value)
            .map_err(|_| ApplicationCorrectionDerivationDenial::InvalidPreImage)?;
        Ok(Revise {
            key: original.key.clone(),
            samples: vec![],
            reference: id.0.to_vec(),
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("restore:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
    fn visit_reapplication<V: ApplicationReapplicationVisitor<Schema>>(
        dispatch: ApplicationReapplicationDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
    fn reapplication_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("reapply:{key}"))
    }
}
