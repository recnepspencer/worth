pub use super::*;
pub use worth_query_host::facade::declaration::{application_aftermath::*, application_query::*};

pub struct RegisterReviewedRetention;
impl ApplicationMutationInputScope<Schema, SetRetentionBinding> for SetRetentionInput {
    fn scope_for_binding(
        &self,
    ) -> <SetRetentionBinding as ApplicationMutationBinding<Schema>>::ScopeBinding {
        SetRetentionIntent {
            input: self.clone(),
        }
        .scope_binding()
    }
}
impl worth_query_decl::facade::application_schema::fragment::Registration<Schema> for RegisterReviewedRetention {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<Schema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                CorrectableRetention::reference()
            .definition()
            .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<CorrectableRetentionBinding>())
    }
}
