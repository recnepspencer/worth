//! Complete retired truth and created-output custody supply opposite native inverses.
use super::*;
pub struct UndoRetirement;
pub struct UndoRestoration;
macro_rules! input_recovery {
    ($source:ty, $inverse:ty) => {
        fn recover_source_input(bytes: &[u8], expected: CanonicalDigestId, bounds: ValueDecodeBounds) -> Result<ApplicationEncodedInput<<Self::Source as ApplicationMutationBinding<Schema>>::InputBinding>, ApplicationRetainedInputDenial> {
            ApplicationEncodedInput::decode_retained_with_seed(bytes, expected, bounds, CanonicalValueSeed::<$source>::new(bounds))
        }
        fn recover_inverse_input(bytes: &[u8], expected: CanonicalDigestId, bounds: ValueDecodeBounds) -> Result<ApplicationEncodedInput<<ApplicationInverseBinding<Schema, Self> as ApplicationMutationBinding<Schema>>::InputBinding>, ApplicationRetainedInputDenial> {
            ApplicationEncodedInput::decode_retained_with_seed(bytes, expected, bounds, CanonicalValueSeed::<$inverse>::new(bounds))
        }
        fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> { Ok(format!("inverse:{key}")) }
        fn visit<V: ApplicationCorrectionVisitor<Schema>>(dispatch: ApplicationCorrectionDispatch<Schema, Self, V>) -> V::Output { dispatch.conventional() }
        fn reapplication_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> { Ok(format!("reapply:{key}")) }
        fn visit_reapplication<V: ApplicationReapplicationVisitor<Schema>>(dispatch: ApplicationReapplicationDispatch<Schema, Self, V>) -> V::Output { dispatch.conventional() }
    };
}
impl ApplicationCorrectionBinding<Schema> for UndoRetirement {
    const IDENTITY: &'static str = "native-record-patterns.complete-retire-correction.v1";
    type Source = retire::Binding;
    type InverseIntent = restore::Input;
    type Disclosure = query::Binding;
    input_recovery!(retire::Input, restore::Input);
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Schema, retire::Retire, retire::Input, true, false>
    {
        retire::Retire::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &retire::Input,
    ) -> Result<ApplicationQueryParameterSet<query::Query>, ApplicationCorrectionDerivationDenial>
    {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::retired_fields_for_output(
                retire::Retired::DESCRIPTOR,
                loci(),
                4096,
            ),
        )
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
        Some(ApplicationAuthoredCorrection::restore_retired::<
            Self,
            Family,
            retire::Retired,
            restore::Created,
        >())
    }
    fn derive_correction(
        original: &retire::Input,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<restore::Input, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        use ApplicationCorrectionFieldPreImage as State;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(image) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let role = retire::Retired::DESCRIPTOR;
        image.output_entity(&role).ok_or(MissingPreImage)?;
        let state = |locus| image.field_state_for_output(&role, &locus);
        let decode = |value| U64ApplicationValueBinding::decode(value).map_err(|_| InvalidPreImage);
        let key = match state(DeclaredPreImageLocus::from_field(Key::at("Key"))) {
            State::Value(value) => {
                StringApplicationValueBinding::decode(value).map_err(|_| InvalidPreImage)?
            }
            _ => return Err(MissingPreImage),
        };
        if key != original.key {
            return Err(InvalidPreImage);
        }
        let first = state(DeclaredPreImageLocus::from_field(FirstValue::at(
            "FirstValue",
        )));
        let second = state(DeclaredPreImageLocus::from_field(SecondValue::at(
            "SecondValue",
        )));
        let required = state(DeclaredPreImageLocus::from_field(SecondRequired::at(
            "SecondRequired",
        )));
        let third = state(DeclaredPreImageLocus::from_field(ThirdValue::at(
            "ThirdValue",
        )));
        let row = match (first, second, required, third) {
            (
                State::Value(first),
                State::AbsentAspect,
                State::AbsentAspect,
                State::AbsentAspect,
            ) => query::Row {
                class: 1,
                first: Some(decode(first)?),
                second: None,
                required: None,
                third: None,
            },
            (
                State::AbsentAspect,
                State::Value(second),
                State::Value(required),
                State::AbsentAspect,
            ) => query::Row {
                class: 2,
                first: None,
                second: Some(decode(second)?),
                required: Some(decode(required)?),
                third: None,
            },
            (
                State::AbsentAspect,
                State::AbsentAspect,
                State::AbsentAspect,
                State::Value(third),
            ) => query::Row {
                class: 3,
                first: None,
                second: None,
                required: None,
                third: Some(decode(third)?),
            },
            _ => return Err(MissingPreImage),
        };
        let (annotation, note) = match (
            state(DeclaredPreImageLocus::from_field(AnnotationValue::at(
                "Value",
            ))),
            state(DeclaredPreImageLocus::from_field(AnnotationNote::at(
                "Note",
            ))),
        ) {
            (State::Value(value), State::AbsentField) => (Some(decode(value)?), None),
            (State::Value(value), State::Value(note)) => {
                (Some(decode(value)?), Some(decode(note)?))
            }
            (State::AbsentAspect, State::AbsentAspect) => (None, None),
            _ => return Err(MissingPreImage),
        };
        Ok(restore::Input {
            image: Image {
                key,
                row,
                annotation,
                note,
            },
        })
    }
}
impl ApplicationCorrectionBinding<Schema> for UndoRestoration {
    const IDENTITY: &'static str = "native-record-patterns.complete-restore-correction.v1";
    type Source = restore::Binding;
    type InverseIntent = retire::Input;
    type Disclosure = query::Binding;
    input_recovery!(restore::Input, retire::Input);
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Schema, restore::Restore, restore::Input, true, false>
    {
        restore::Restore::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &restore::Input,
    ) -> Result<ApplicationQueryParameterSet<query::Query>, ApplicationCorrectionDerivationDenial>
    {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_create_inverse_binding::<Self>()
    }
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
        Some(ApplicationAuthoredCorrection::retire_created::<
            Self,
            Family,
            restore::Created,
            retire::Retired,
        >())
    }
    fn derive_correction(
        original: &restore::Input,
        _: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<retire::Input, ApplicationCorrectionDerivationDenial> {
        Ok(retire::Input {
            key: original.image.key.clone(),
            collateral_third_observation: false,
        })
    }
}
