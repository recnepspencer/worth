//! Complete retired truth and created-output custody supply opposite native inverses.
use super::*;
pub struct RegisterRetirement;
pub struct RegisterRestoration;
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
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for RegisterRetirement {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                retire::Retire::reference()
            .definition()
            .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<retire::Binding>())
    }
}
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for RegisterRestoration {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                restore::Restore::reference()
            .definition()
            .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<restore::Binding>())
    }
}
