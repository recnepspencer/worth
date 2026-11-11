//! Correction derives from declared original output fields after current disclosure.
use super::{query, schema::*, source_output_model::*};
use worth_query_host::facade::declaration::{
    application_aftermath::*, application_query::*, application_schema::*,
};
pub struct RecordIssued;
pub fn key_field() -> ApplicationCorrectionOutputField<NumericSchema> {
    ApplicationCorrectionOutput::<IssueBinding, Actual>::field(Key::at("Key"))
}
pub fn payload_field() -> ApplicationCorrectionOutputField<NumericSchema> {
    ApplicationCorrectionOutput::<IssueBinding, Actual>::field(Primary::at("Primary"))
}
pub static DERIVATIONS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub fn derivations() -> usize {
    DERIVATIONS.load(std::sync::atomic::Ordering::Relaxed)
}
impl ApplicationCorrectionBinding<NumericSchema> for RecordIssued {
    const IDENTITY: &'static str = "source-output.record-issued.v1";
    type Source = IssueBinding;
    type InverseIntent = Record;
    type Disclosure = query::ReadBinding;

    fn recover_source_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<NumericSchema>>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    >{
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<Self::Source as worth_query_decl::facade::application_operation::ApplicationMutationBinding<NumericSchema>>::Input>::new(bounds),
        )
    }

    fn recover_inverse_input(
        bytes: &[u8],
        expected: worth_query_decl::facade::application_schema::CanonicalDigestId,
        bounds: worth_query_decl::facade::application_schema::ValueDecodeBounds,
    ) -> Result<
        worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            <worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<
                NumericSchema,
                Self,
            > as worth_query_decl::facade::application_operation::ApplicationMutationBinding<
                NumericSchema,
            >>::InputBinding,
        >,
        worth_query_decl::facade::application_operation::ApplicationRetainedInputDenial,
    > {
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained_with_seed(
            bytes, expected, bounds,
            worth_query_decl::facade::application_schema::CanonicalValueSeed::<<worth_query_decl::facade::application_aftermath::ApplicationInverseBinding<NumericSchema, Self> as worth_query_decl::facade::application_operation::ApplicationMutationBinding<NumericSchema>>::Input>::new(bounds),
        )
    }
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<NumericSchema, IssueOperation, Issue, true, false>
    {
        IssueOperation::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &Issue,
    ) -> Result<ApplicationQueryParameterSet<query::ReadQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(query::parameters())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<NumericSchema> {
        DeclaredCorrectionMechanism::compensation_binding::<Self>(
            DeclaredAftermathPostcondition::BusinessPostcondition {
                identity: "original-issued-value-recorded".to_owned(),
            },
        )
    }
    fn source_output_fields() -> Vec<ApplicationCorrectionOutputField<NumericSchema>> {
        vec![key_field(), payload_field()]
    }
    fn derive_correction(
        original: &Issue,
        material: ApplicationCorrectionSourceMaterial<'_, NumericSchema>,
    ) -> Result<Record, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning;
        DERIVATIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let ApplicationCorrectionSourceMaterial::Compensation(fields) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let key = StringApplicationValueBinding::decode(
            fields
                .field(&key_field())
                .map_err(|_| UnsupportedSourceMeaning)?,
        )
        .map_err(|_| UnsupportedSourceMeaning)?;
        let payload = InlineCounterBinding::decode(
            fields
                .field(&payload_field())
                .map_err(|_| UnsupportedSourceMeaning)?,
        )
        .map_err(|_| UnsupportedSourceMeaning)?
        .0;
        Ok(Record {
            key: original.key.clone(),
            original_key: key,
            payload,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("record-issued:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<NumericSchema>>(
        dispatch: ApplicationCorrectionDispatch<NumericSchema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
