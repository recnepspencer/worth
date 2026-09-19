#[cfg(any(test, feature = "certification-query-lookup"))]
use worth_query_declaration::facade::application_query::ApplicationQueryReference;
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBindingDescriptor, ErasedApplicationQueryDefinition,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ApplicationQueryBindingKey {
    name: String,
    query_type: WorthQueryPortableTypeIdentity,
    parameter_type: WorthQueryPortableTypeIdentity,
    result_type: WorthQueryPortableTypeIdentity,
    scope_type: WorthQueryPortableTypeIdentity,
}

impl ApplicationQueryBindingKey {
    pub(crate) fn from_definition(definition: &ErasedApplicationQueryDefinition) -> Self {
        Self {
            name: definition.name().to_owned(),
            query_type: definition.query_identity(),
            parameter_type: definition.parameter_identity(),
            result_type: definition.result_identity(),
            scope_type: definition.scope_identity(),
        }
    }

    #[cfg(any(test, feature = "certification-query-lookup"))]
    pub(crate) fn from_reference<Schema, Query, Parameters, QueryResult, Scope>(
        reference: &ApplicationQueryReference<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> Self {
        Self {
            name: reference.name().to_owned(),
            query_type: reference.query_type(),
            parameter_type: reference.parameter_type(),
            result_type: reference.result_type(),
            scope_type: reference.scope_type(),
        }
    }

    pub(crate) fn from_binding(descriptor: &ApplicationQueryBindingDescriptor) -> Self {
        Self {
            name: descriptor.query_name().to_owned(),
            query_type: descriptor.query_identity().clone(),
            parameter_type: descriptor.parameter_identity().clone(),
            result_type: descriptor.result_identity().clone(),
            scope_type: descriptor.scope_identity().clone(),
        }
    }
}
