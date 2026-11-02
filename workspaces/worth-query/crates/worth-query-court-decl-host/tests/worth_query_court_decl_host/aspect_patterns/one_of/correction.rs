use super::*;
use worth_query_decl::facade::application_aftermath::*;
pub struct Restore;
impl ApplicationCorrectionBinding<Schema> for Restore {
    const IDENTITY: &'static str = "native-record-patterns.restore.v1";
    type Source = authoring::Binding;
    type InverseIntent = restore_first::Restore;
    type Disclosure = query::Binding;
    fn source_definition() -> ApplicationOperationDefinitionBuilder<
        Schema,
        authoring::Transition,
        authoring::Edit,
        true,
        false,
    > {
        authoring::Transition::reference()
            .definition()
            .no_external_effect()
    }
    fn disclosure_parameters(
        _: &authoring::Edit,
    ) -> Result<ApplicationQueryParameterSet<query::Query>, ApplicationCorrectionDerivationDenial>
    {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new(
                [
                    DeclaredPreImageLocus::from_field(Class::reference()),
                    DeclaredPreImageLocus::from_field(FirstValue::reference()),
                ],
                // Two UInt64 atoms: each has eight payload bytes and one scalar tag.
                18,
            )
            .unwrap(),
        )
    }
    fn derive_correction(
        original: &authoring::Edit,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<restore_first::Restore, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::{
            InvalidPreImage, MissingPreImage, UnsupportedSourceMeaning,
        };
        let ApplicationCorrectionSourceMaterial::RecordedInverse(preimage) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let decode = |locus| {
            preimage
                .field(&locus)
                .map(U64ApplicationValueBinding::decode)
                .transpose()
                .map_err(|_| InvalidPreImage)
        };
        let row = query::Row {
            class: decode(DeclaredPreImageLocus::from_field(Class::reference()))?
                .ok_or(MissingPreImage)?,
            first: Some(
                decode(DeclaredPreImageLocus::from_field(FirstValue::reference()))?
                    .ok_or(MissingPreImage)?,
            ),
            second: None,
            required: None,
            third: None,
        };
        // The source operation admits Class1 only. Its permanently installed
        // one-of rule certifies that every other class aspect was absent.
        // This is a directional inverse, not absent/unmutated-field retention.
        if row.class != 1 {
            return Err(InvalidPreImage);
        }
        Ok(restore_first::Restore {
            key: original.key.clone(),
            row,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("restore:{key}"))
    }
    fn reapplication_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("reapply:{key}"))
    }
    fn visit_reapplication<V: ApplicationReapplicationVisitor<Schema>>(
        dispatch: ApplicationReapplicationDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
    fn visit<V: ApplicationCorrectionVisitor<Schema>>(
        dispatch: ApplicationCorrectionDispatch<Schema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}
