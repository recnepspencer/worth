//! Recorded XOR inversion retains the prior value and admits its exact after-image.
use super::*;
use worth_query_decl::facade::{application_aftermath::*, application_query::*};

pub struct RegisterEntryEdit<Schema>(PhantomData<fn() -> Schema>);

impl<Schema: TopologySchemaBinding> worth_query_decl::facade::application_schema::fragment::Registration<Schema> for RegisterEntryEdit<Schema> {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                CorrectEntryValue::reference()
            .definition()
            .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<EntryCorrectionBinding<Schema>>())
    }
}
