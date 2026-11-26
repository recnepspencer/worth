use crate::application_query::{
    ApplicationQueryBinding, ApplicationQueryDefinitionBuilder, ApplicationQueryScopeBinding,
};

use super::{
    ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember, ApplicationStructuredValueBinding,
};

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

    /// Declare query meaning and its matching request binding in one registration.
    ///
    /// If an identical binding descriptor was already registered (for example as
    /// a correction's disclosure query binding), that registration is reused;
    /// conflicting binding meanings and duplicate query definitions are still
    /// refused at build time. A binding whose input field scopes the query's
    /// own root must name a key field (`unique` or `read_only`); `build()`
    /// refuses a mutable, non-unique one with `RootScopedQueryRequiresKeyField`.
    pub fn bound_application_query<Binding>(
        self,
        definition: ApplicationQueryDefinitionBuilder<
            Schema,
            Binding::Query,
            <Binding::ParameterBinding as ApplicationStructuredValueBinding>::Value,
            <Binding::ResultBinding as ApplicationStructuredValueBinding>::Value,
            <Binding::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
    ) -> Self
    where
        Schema: super::ApplicationSchema,
        Binding: ApplicationQueryBinding<Schema>,
    {
        let mut builder = self.application_query(definition);
        let descriptor = Binding::descriptor();
        builder.root_scope_keys.expect(descriptor.clone());
        if builder
            .member_provenance
            .query_bindings()
            .contains(&descriptor)
        {
            builder
        } else {
            builder.application_query_binding::<Binding>()
        }
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
