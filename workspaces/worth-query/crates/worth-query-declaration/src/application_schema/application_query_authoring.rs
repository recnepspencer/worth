use crate::application_query::{ApplicationQueryBinding, ApplicationQueryDefinition};

use super::{ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Register separate history meaning and its exact original SubjectRead binding.
    pub fn application_history_query<History>(
        mut self,
        definition: crate::application_query::ApplicationHistoryQueryDefinition<Schema, History>,
    ) -> Self
    where
        Schema: super::ApplicationSchema,
        History: crate::application_query::ApplicationHistoryQueryBinding<Schema>,
    {
        let subject = History::SubjectRead::descriptor();
        if !self.member_provenance.query_bindings().contains(&subject) {
            self.member_provenance.register_query_binding(subject);
        }
        self.member_provenance
            .register_history::<History, History::SubjectRead>(
                History::IDENTITY,
                History::PORTABLE_TYPE_IDENTITY,
            );
        self.push_member(ApplicationSchemaMember::ApplicationHistoryQuery {
            definition: definition.into_erased(),
        })
    }
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
