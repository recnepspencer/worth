//! Restore real recorded member images and the owner's prior field value.
use super::*;
pub struct Revision;
impl ApplicationCorrectionBinding<Schema> for Revision {
    const IDENTITY: &'static str = "owned-outputs.member-retarget-inverse";
    type Source = Binding;
    type InverseIntent = Intent;
    type Disclosure = query::model::Binding;
    fn recover_source_input(
        bytes: &[u8],
        expected: CanonicalDigestId,
        bounds: ValueDecodeBounds,
    ) -> Result<ApplicationEncodedInput<Input>, ApplicationRetainedInputDenial> {
        ApplicationEncodedInput::decode_retained_with_seed(
            bytes,
            expected,
            bounds,
            CanonicalValueSeed::<Intent>::new(bounds),
        )
    }
    fn recover_inverse_input(
        bytes: &[u8],
        expected: CanonicalDigestId,
        bounds: ValueDecodeBounds,
    ) -> Result<ApplicationEncodedInput<Input>, ApplicationRetainedInputDenial> {
        Self::recover_source_input(bytes, expected, bounds)
    }
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Schema, Revise, Intent, true, false> {
        Revise::reference().definition().no_external_effect()
    }
    fn disclosure_parameters(
        _: &Intent,
    ) -> Result<
        ApplicationQueryParameterSet<query::model::Query>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("undo:{key}"))
    }
    fn reapplication_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("reapply:{key}"))
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
    fn authored_identity() -> Option<ApplicationAuthoredCorrection<Schema>> {
        Some(
            ApplicationAuthoredCorrection::live_owner_members::<
                Self,
                Family,
                Leaving,
                Arriving,
                _,
                _,
            >(InputOwner::reference())
            .and_live_owner_members::<Self, Family, Arriving, Leaving, _, _>(
                InputOwner::reference(),
            ),
        )
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::changed_fields_for_output(
                Owner::DESCRIPTOR,
                [
                    DeclaredPreImageLocus::from_field(ModelValue::at("ModelValue")),
                    DeclaredPreImageLocus::from_field(InputKey::at("InputKey")),
                    DeclaredPreImageLocus::from_field(InputValue::at("InputValue")),
                ],
                4096,
            )
            .with_relations([DeclaredPreImageRelationLocus::from_relation(
                InputOwner::reference(),
            )]),
        )
    }
    fn derive_correction(
        original: &Intent,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<Intent, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(prior) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let ApplicationCorrectionFieldPreImage::Value(value) = prior.field_state_for_output(
            &Owner::DESCRIPTOR,
            &DeclaredPreImageLocus::from_field(ModelValue::at("ModelValue")),
        ) else {
            return Err(MissingPreImage);
        };
        let value = U64ApplicationValueBinding::decode(value).map_err(|_| InvalidPreImage)?;
        let mut create = Vec::new();
        for key in &original.retire {
            let field = |locus| {
                prior
                    .field_for_family_member(&Leaving::DESCRIPTOR, key, &locus)
                    .ok_or(MissingPreImage)
            };
            create.push(Row {
                key: StringApplicationValueBinding::decode(field(
                    DeclaredPreImageLocus::from_field(InputKey::at("InputKey")),
                )?)
                .map_err(|_| InvalidPreImage)?,
                value: U64ApplicationValueBinding::decode(field(
                    DeclaredPreImageLocus::from_field(InputValue::at("InputValue")),
                )?)
                .map_err(|_| InvalidPreImage)?,
            });
        }
        Ok(Intent {
            owner: original.owner.clone(),
            value,
            retire: original.create.iter().map(|row| row.key.clone()).collect(),
            create,
        })
    }
}
