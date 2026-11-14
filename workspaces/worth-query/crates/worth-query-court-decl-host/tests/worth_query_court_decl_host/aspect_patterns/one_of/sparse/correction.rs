//! The real preserved role discloses only original changed fields, not a complete image.
use super::*;
pub struct Correction;
impl ApplicationCorrectionBinding<Schema> for Correction {
    const IDENTITY: &'static str = "native-record-patterns.sparse.correction.v1";
    type Source = Binding;
    type InverseIntent = Input;
    type Disclosure = query::Binding;
    fn recover_source_input(
        bytes: &[u8],
        expected: CanonicalDigestId,
        bounds: ValueDecodeBounds,
    ) -> Result<ApplicationEncodedInput<InputValue>, ApplicationRetainedInputDenial> {
        ApplicationEncodedInput::decode_retained_with_seed(
            bytes,
            expected,
            bounds,
            CanonicalValueSeed::<Input>::new(bounds),
        )
    }
    fn recover_inverse_input(
        bytes: &[u8],
        expected: CanonicalDigestId,
        bounds: ValueDecodeBounds,
    ) -> Result<ApplicationEncodedInput<InputValue>, ApplicationRetainedInputDenial> {
        Self::recover_source_input(bytes, expected, bounds)
    }
    fn source_definition() -> ApplicationOperationDefinitionBuilder<Schema, Edit, Input, true, false>
    {
        Edit::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Input,
    ) -> Result<ApplicationQueryParameterSet<query::Query>, ApplicationCorrectionDerivationDenial>
    {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::changed_fields_for_output(Kept::DESCRIPTOR, loci(), 4096),
        )
    }
    fn derive_correction(
        original: &Input,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<Input, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        use ApplicationCorrectionFieldPreImage as State;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(image) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let read = |locus| image.field_state_for_output(&Kept::DESCRIPTOR, &locus);
        assert_eq!(
            read(DeclaredPreImageLocus::from_field(Key::at("Key"))),
            State::Uncaptured,
            "merely reading an eligible unchanged field cannot create retained original material"
        );
        let first = read(DeclaredPreImageLocus::from_field(FirstValue::at(
            "FirstValue",
        )));
        let second = read(DeclaredPreImageLocus::from_field(SecondValue::at(
            "SecondValue",
        )));
        let required = read(DeclaredPreImageLocus::from_field(SecondRequired::at(
            "SecondRequired",
        )));
        let decode = |value| U64ApplicationValueBinding::decode(value).map_err(|_| InvalidPreImage);
        let case = match (first, second, required) {
            (State::Value(first), State::AbsentAspect, State::AbsentAspect) => Case::First {
                value: decode(first)?,
            },
            (State::AbsentAspect, State::Value(second), State::Value(required)) => Case::Second {
                value: decode(second)?,
                required: decode(required)?,
            },
            _ => return Err(MissingPreImage),
        };
        Ok(Input {
            key: original.key.clone(),
            case,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("sparse:inverse:{key}"))
    }
    fn reapplication_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("sparse:reapply:{key}"))
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
}
