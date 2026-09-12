use crate::application_query::{ApplicationQueryBinding, ApplicationQueryDefinition};

use super::{ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Registers one complete typed query binding for installation.
    pub fn application_query_binding<Binding>(mut self) -> Self
    where
        Schema: super::ApplicationSchema,
        Binding: ApplicationQueryBinding<Schema>,
    {
        self.member_provenance
            .register_query_binding(Binding::descriptor());
        self
    }

    /// Declares immutable application-query meaning as part of this package.
    ///
    /// Installed runtimes resolve typed query references against this retained
    /// member; callers cannot add query meaning after package installation.
    pub fn application_query<Query, Parameters, QueryResult, Scope>(
        self,
        definition: ApplicationQueryDefinition<Schema, Query, Parameters, QueryResult, Scope>,
    ) -> Self {
        self.push_member(ApplicationSchemaMember::ApplicationQuery {
            definition: definition.into_erased(),
        })
    }
}
