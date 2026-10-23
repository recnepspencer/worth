//! The domain inverse restores the original chosen field from captured native values.
use super::{edit::*, other_edit::OtherEdit, query, schema::*};
use worth_query_host::facade::declaration::{
    application_aftermath::*, application_query::*, application_schema::*,
};

pub struct RecordedEdit;
impl ApplicationCorrectionBinding<NumericSchema> for RecordedEdit {
    const IDENTITY: &'static str = "numeric-correction.recorded-edit.v1";
    type Source = EditBinding;
    type InverseIntent = OtherEdit;
    type Disclosure = query::ReadBinding;

    fn disclosure_parameters(
        _: &Edit,
    ) -> Result<ApplicationQueryParameterSet<query::ReadQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(query::parameters())
    }
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<NumericSchema, EditNumber, Edit, true, false> {
        EditNumber::reference().definition().no_external_effect()
    }
    fn preimage_demand() -> DeclaredPreImageDemand<NumericSchema> {
        // Capture stores the binding's complete AspectValue representation.
        let encoded_width = U64ApplicationValueBinding::encode(&u64::MAX)
            .unwrap()
            .semantic_byte_width();
        DeclaredPreImageDemand::new(
            [DeclaredPreImageLocus::from_field(Primary::reference())],
            encoded_width,
        )
        .unwrap()
    }
    fn derive_inverse(
        original: &Edit,
        preimage: &impl ApplicationCorrectionPreImage<NumericSchema>,
    ) -> Result<OtherEdit, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::{InvalidPreImage, MissingPreImage};
        let value = match original.value {
            Value::Primary(_) => Value::Primary(
                U64ApplicationValueBinding::decode(
                    preimage
                        .field(&DeclaredPreImageLocus::from_field(Primary::reference()))
                        .ok_or(MissingPreImage)?,
                )
                .map_err(|_| InvalidPreImage)?,
            ),
            _ => return Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning),
        };
        Ok(OtherEdit {
            key: original.key.clone(),
            value,
        })
    }
    fn inverse_key(key: &str) -> String {
        key.to_owned()
    }
    fn visit<Visitor: ApplicationCorrectionVisitor<NumericSchema>>(
        dispatch: ApplicationCorrectionDispatch<NumericSchema, Self, Visitor>,
    ) -> Visitor::Output {
        dispatch.conventional()
    }
}
