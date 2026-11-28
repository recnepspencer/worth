//! Register the Catalog contribution's note and movement operations.
use super::*;
pub struct RegisterNote;
pub struct RegisterMover;
pub struct RegisterMotion;
impl worth_query_decl::facade::application_schema::fragment::Registration<Catalog> for RegisterNote {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Catalog>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                note::WriteNote::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<note::Binding>())
    }
}
impl worth_query_decl::facade::application_schema::fragment::Registration<Catalog> for RegisterMover {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Catalog>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                mover::MoveName::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<mover::Binding>())
    }
}

impl worth_query_decl::facade::application_schema::fragment::Registration<Catalog> for RegisterMotion {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Catalog>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Catalog>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                motion::MoveVertex::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<motion::Binding>())
    }
}
