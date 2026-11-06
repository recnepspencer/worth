use super::*;
pub struct Restore;
impl ApplicationCorrectionBinding<Schema> for Restore {
    const IDENTITY: &'static str = "source-subgraph.restore-value.v1";
    type Source = edit::source::Binding;
    type InverseIntent = edit::source::Intent;
    type Disclosure = graph_read::source::Binding;
    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        Schema,
        edit::source::PatchSource,
        edit::source::Intent,
        true,
        false,
    > {
        edit::source::PatchSource::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &edit::source::Intent,
    ) -> Result<
        ApplicationQueryParameterSet<graph_read::source::Query>,
        ApplicationCorrectionDerivationDenial,
    > {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [DeclaredPreImageLocus::from_field(SourceValue::at(
                    "SourceValue",
                ))],
                9,
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &edit::source::Intent,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<edit::source::Intent, ApplicationCorrectionDerivationDenial> {
        let ApplicationCorrectionSourceMaterial::RecordedInverse(prior) = material else {
            return Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning);
        };
        let value = prior
            .field(&DeclaredPreImageLocus::from_field(SourceValue::at(
                "SourceValue",
            )))
            .ok_or(ApplicationCorrectionDerivationDenial::MissingPreImage)?;
        let value = U64ApplicationValueBinding::decode(value)
            .map_err(|_| ApplicationCorrectionDerivationDenial::InvalidPreImage)?;
        Ok(edit::source::Intent(edit::Edit {
            key: original.0.key.clone(),
            value,
        }))
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("restore:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
