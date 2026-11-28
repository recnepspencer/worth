use super::*;
pub struct RegisterEdit;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for RegisterEdit {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                edit::source::PatchSource::reference()
            .definition()
            .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<edit::source::Binding>())
    }
}
