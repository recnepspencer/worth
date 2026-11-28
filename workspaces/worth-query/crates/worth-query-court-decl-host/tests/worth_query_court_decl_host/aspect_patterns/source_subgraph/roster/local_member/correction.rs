//! The inverse returns every retained member and note of the original typed local intent.
use super::*;
pub struct RegisterDeletion;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for RegisterDeletion {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                DeleteMember::reference().definition().no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<DeleteBinding>())
    }
}
