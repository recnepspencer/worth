//! Size edits use the existing recorded inverse and retained reapplication contract.
use super::{source_control::*, *};
pub struct RegisterResize;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for RegisterResize {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                ControlSource::reference().definition().no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<Binding>())
    }
}
