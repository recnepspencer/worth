//! Size edits use the existing recorded inverse and retained reapplication contract.
use super::{source_control::*, *};
pub struct Resize;
impl ApplicationCorrectionBinding<Schema> for Resize {
    const IDENTITY: &'static str = "succession.resize.inverse";
    type Source = Binding;
    type InverseIntent = Control;
    type Disclosure = ReadBinding;

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
        worth_query_decl::facade::application_operation::ApplicationEncodedInput::decode_retained(
            bytes, expected, bounds,
        )
    }
    fn disclosure_parameters(
        _: &Control,
    ) -> Result<ApplicationQueryParameterSet<Query>, ApplicationCorrectionDerivationDenial> {
        Ok(ApplicationQueryParameterSet::new())
    }
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<Schema, ControlSource, Control, true, false> {
        ControlSource::reference().definition().no_external_effect()
    }
    fn mechanism() -> DeclaredCorrectionMechanism<Schema> {
        DeclaredCorrectionMechanism::recorded_inverse_binding::<Self>(
            DeclaredPreImageDemand::new([DeclaredPreImageLocus::from_field(Width::reference())], 9)
                .unwrap(),
        )
    }
    fn derive_correction(
        original: &Control,
        material: ApplicationCorrectionSourceMaterial<'_, Schema>,
    ) -> Result<Control, ApplicationCorrectionDerivationDenial> {
        use ApplicationCorrectionDerivationDenial::*;
        let ApplicationCorrectionSourceMaterial::RecordedInverse(fields) = material else {
            return Err(UnsupportedSourceMeaning);
        };
        let width = U64ApplicationValueBinding::decode(
            fields
                .field(&DeclaredPreImageLocus::from_field(Width::reference()))
                .ok_or(MissingPreImage)?,
        )
        .map_err(|_| InvalidPreImage)?;
        Ok(Control {
            key: original.key.clone(),
            width,
        })
    }
    fn inverse_key(key: &str) -> Result<u64, ApplicationValueEncodeDenial> {
        Ok(key
            .bytes()
            .fold(500u64, |v, b| v.wrapping_mul(31).wrapping_add(b as u64)))
    }
    fn reapplication_key(key: &str) -> Result<u64, ApplicationValueEncodeDenial> {
        Self::inverse_key(key)
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
