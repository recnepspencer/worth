//! Actual recorded inverse declarations for the Catalog contribution.
use super::*;
use worth_query_decl::facade::application_query::*;
pub struct RegisterNote;
pub struct RegisterMover;
pub struct RegisterMotion;
const PREIMAGE_BYTES: usize = 4096; // An explicit fixture retention ceiling; oversized values refuse.
fn key(value: &str, identity: &'static str) -> Result<u64, ApplicationValueEncodeDenial> {
    value.parse().map_err(|_| ApplicationValueEncodeDenial::CodecRejected {
        binding_identity: worth_query_decl::facade::portable_identity::WorthQueryPortableTypeIdentity::declared(identity),
        reason: "recorded catalog correction keys are unsigned decimal integers",
    })
}
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
