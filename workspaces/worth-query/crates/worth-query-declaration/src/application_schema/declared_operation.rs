//! Schema projection of the access facts declared once on an operation.
use super::{
    ApplicationOperationDefinition, ApplicationOperationMarkerIdentity,
    ApplicationSchemaDeclarationBuilder, ApplicationStructuredValueBinding,
};

/// Typed access supplied by `worth_query_operation!`'s `access` clause.
/// This is descriptive meaning; native installation and admission still enforce authority.
///
/// ```
/// use worth_query_declaration::facade::application_schema::*;
/// use worth_query_declaration::{worth_query_entity, worth_query_operation,
///     worth_query_structured_value_binding};
/// struct Schema;
/// worth_query_entity!(Record for Schema);
/// worth_query_structured_value_binding!(Input for () { identity: "example.operation.input.v1" });
/// worth_query_operation!(Read for Schema, input Input; access { read_entities: [Record]; });
/// fn accepts_read<T: OperationReads<Read>>() {}
/// accepts_read::<Record>();
/// ```
///
/// An undeclared member gains no capability from another member's access:
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_schema::*;
/// use worth_query_declaration::{worth_query_entity, worth_query_operation,
///     worth_query_structured_value_binding};
/// struct Schema;
/// worth_query_entity!(Record for Schema);
/// worth_query_entity!(Unlisted for Schema);
/// worth_query_structured_value_binding!(Input for () { identity: "example.operation.input.v1" });
/// worth_query_operation!(Read for Schema, input Input; access { read_entities: [Record]; });
/// fn accepts_read<T: OperationReads<Read>>() {}
/// accepts_read::<Unlisted>();
/// ```
pub trait ApplicationDeclaredOperation<Schema>: ApplicationOperationMarkerIdentity<Schema> {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema>;
}

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Registers one operation with explicit effect/aftermath policy and its declared access.
    /// No read-work ceiling is inferred from the access set.
    pub fn declared_operation<Operation>(
        self,
        definition: ApplicationOperationDefinition<
            Schema,
            Operation,
            <Operation::InputBinding as ApplicationStructuredValueBinding>::Value,
        >,
    ) -> Self
    where
        Operation: ApplicationDeclaredOperation<Schema> + 'static,
        <Operation::InputBinding as ApplicationStructuredValueBinding>::Value: 'static,
    {
        Operation::declare_access(self.operation(definition))
    }
}

#[cfg(test)]
mod tests;
