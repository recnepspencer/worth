//! The inverse is the same revision: changed owner fields and moved members swap back.
use super::*;
use local_member::Returned;
pub struct RegisterRevision;
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for RegisterRevision {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                ReviseOwner::reference().definition().no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<ReviseBinding>())
    }
}
