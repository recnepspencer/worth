use crate::application_query::{ApplicationQueryBinding, ApplicationQueryDefinitionBuilder};

use super::{ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    #[doc(hidden)]
    pub fn in_feature(mut self, owner: &str, members: impl FnOnce(Self) -> Self) -> Self {
        let previous = self.feature_owner.replace(owner.to_owned());
        let mut schema = members(self);
        schema.feature_owner = previous;
        schema
    }

    /// Attach optional inspection metadata to the complete existing query binding.
    pub fn application_query_inspection<Binding>(
        mut self,
        definition: crate::application_query::ApplicationQueryInspectionDefinition<Schema, Binding>,
    ) -> Self
    where
        Schema: super::ApplicationSchema,
        Binding: ApplicationQueryBinding<Schema>,
    {
        let descriptor = Binding::descriptor();
        if !self
            .member_provenance
            .query_bindings()
            .contains(&descriptor)
        {
            self.member_provenance.register_query_binding(descriptor);
        }
        self.push_member(ApplicationSchemaMember::ApplicationQueryInspection {
            definition: definition.into_erased(),
        })
    }
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
        mut self,
        definition: ApplicationQueryDefinitionBuilder<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
    ) -> Self {
        let query = definition.name().to_owned();
        let mut definition = match definition.build() {
            Ok(definition) => definition,
            Err(cause) => {
                self.refusal.get_or_insert(
                    super::ApplicationSchemaDeclarationDenial::InvalidQueryDefinition {
                        owner: self
                            .feature_owner
                            .clone()
                            .unwrap_or_else(|| self.name.to_owned()),
                        query,
                        cause,
                    },
                );
                return self;
            }
        };
        for field in definition.take_field_expectations() {
            self.member_provenance.fragments.expect_field(field);
        }
        if let Some(recipe) = definition.take_managed_index_recipe() {
            self.member_provenance
                .register_managed_index_selection(recipe);
        }
        self.push_member(ApplicationSchemaMember::ApplicationQuery {
            definition: definition.into_erased(),
        })
    }
}
