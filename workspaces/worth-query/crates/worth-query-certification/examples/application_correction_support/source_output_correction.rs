//! Correction derives from declared original output fields after current disclosure.
use super::{query, schema::*, source_output_model::*};
use worth_query_host::facade::declaration::{
    application_aftermath::*, application_query::*, application_schema::*,
};
pub struct RecordIssued;
pub fn key_field() -> ApplicationCorrectionOutputField<NumericSchema> {
    ApplicationCorrectionOutput::<IssueBinding, Actual>::field(Key::reference())
}
pub fn payload_field() -> ApplicationCorrectionOutputField<NumericSchema> {
    ApplicationCorrectionOutput::<IssueBinding, Actual>::field(Primary::reference())
}
static DERIVATIONS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub fn derivations() -> usize {
    DERIVATIONS.load(std::sync::atomic::Ordering::Relaxed)
}
impl ApplicationCorrectionBinding<NumericSchema> for RecordIssued {
    const IDENTITY: &'static str = "source-output.record-issued.v1";
    type Source = IssueBinding;
    type InverseIntent = Record;
    type Disclosure = query::ReadBinding;
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
        .unwrap()
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
        let payload = U64ApplicationValueBinding::decode(
            fields
                .field(&payload_field())
                .map_err(|_| UnsupportedSourceMeaning)?,
        )
        .map_err(|_| UnsupportedSourceMeaning)?;
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
