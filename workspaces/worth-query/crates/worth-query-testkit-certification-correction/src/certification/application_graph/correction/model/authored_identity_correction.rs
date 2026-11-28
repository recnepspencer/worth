//! Both lifecycle directions bind the same family and its authored logical key.
use super::{authored_identity_model::*, query, schema::*};
use worth_query_host::facade::declaration::{
    application_aftermath::*, application_query::*, application_schema::*,
};

pub struct Family;
impl ApplicationAuthoredCorrectionFamily<NumericSchema> for Family {
    const IDENTITY: &'static str = "numeric.authored-number.v1";
    type Entity = Number;
    fn logical_key() -> ApplicationAuthoredLogicalKey<NumericSchema, Number> {
        ApplicationAuthoredLogicalKey::from_field(Key::at("Key"))
    }
}
pub struct RegisterCreate;
pub struct RegisterRetire;
impl worth_query_decl::facade::application_schema::fragment::Registration<NumericSchema> for RegisterCreate {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                CreateOperation::reference()
            .definition()
            .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<CreateBinding>())
    }
}
impl worth_query_decl::facade::application_schema::fragment::Registration<NumericSchema> for RegisterRetire {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                RetireAuthoredEntity::reference()
            .definition()
            .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<RetireBinding>())
    }
}
