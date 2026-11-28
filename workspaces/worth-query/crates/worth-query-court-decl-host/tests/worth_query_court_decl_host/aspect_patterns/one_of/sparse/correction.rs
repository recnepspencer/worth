//! The real preserved role discloses only original changed fields, not a complete image.
use super::*;
pub struct RegisterSparseEdit;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for RegisterSparseEdit {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                Edit::reference().definition().no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<Binding>())
    }
}
