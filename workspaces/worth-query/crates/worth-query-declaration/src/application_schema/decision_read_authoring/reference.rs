//! Operation references carry content identity; a marker alone grants nothing.
use super::super::{
    ApplicationOperationRef, ApplicationSchemaDeclarationBuilder as Builder,
    ApplicationSchemaDeclarationDenial as Denial, ApplicationSchemaMember as Member,
    OperationReads,
};
use super::scope::{ApplicationDecisionReadScopeDefinition, ApplicationDecisionReadScopeMeaning};
use std::marker::PhantomData;

pub struct ApplicationDecisionReadScopeRef<Schema, Scope> {
    identity: String,
    marker: PhantomData<fn() -> (Schema, Scope)>,
}

impl<S, Scope> Clone for ApplicationDecisionReadScopeRef<S, Scope> {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            marker: PhantomData,
        }
    }
}

impl<S, Scope> ApplicationDecisionReadScopeDefinition<S, Scope> {
    pub fn reference(&self) -> Result<ApplicationDecisionReadScopeRef<S, Scope>, Denial> {
        let meaning = ApplicationDecisionReadScopeMeaning::from_untrusted_parts(
            self.targets.clone(),
            self.derived_relations.clone(),
        )?;
        Ok(ApplicationDecisionReadScopeRef {
            identity: meaning.identity().to_owned(),
            marker: PhantomData,
        })
    }
}

impl<S> Builder<S> {
    pub fn decision_read_scope<Scope: 'static>(
        mut self,
        definition: ApplicationDecisionReadScopeDefinition<S, Scope>,
    ) -> Self {
        for recipe in definition.fields {
            self.member_provenance.fragments.expect_field(recipe);
        }
        match ApplicationDecisionReadScopeMeaning::from_untrusted_parts(
            definition.targets,
            definition.derived_relations,
        ) {
            Ok(definition) => {
                let previous = self.member_provenance.decision_read_scopes.insert(
                    std::any::TypeId::of::<Scope>(),
                    definition.identity().to_owned(),
                );
                if previous.is_some_and(|identity| identity != definition.identity()) {
                    self.refusal = Some(Denial::ConflictingDecisionReadScope {
                        scope: std::any::type_name::<Scope>().to_owned(),
                    });
                    return self;
                }
                if self.has_decision_read_scope(&definition) {
                    self
                } else {
                    self.push_member(Member::DecisionReadScope { definition })
                }
            }
            Err(denial) => {
                self.refusal = Some(denial);
                self
            }
        }
    }

    /// Resolve an already authored scope by its actual marker, without rebuilding its contents.
    pub fn decision_read_scope_reference<Scope: 'static>(
        &self,
    ) -> Result<ApplicationDecisionReadScopeRef<S, Scope>, Denial> {
        let identity = self
            .member_provenance
            .decision_read_scopes
            .get(&std::any::TypeId::of::<Scope>())
            .ok_or_else(|| Denial::UndeclaredDecisionReadScope {
                scope: std::any::type_name::<Scope>().to_owned(),
            })?
            .clone();
        Ok(ApplicationDecisionReadScopeRef {
            identity,
            marker: PhantomData,
        })
    }

    /// Sharing requires an explicit Rust capability for this operation as well as installed contents.
    /// ```compile_fail,E0277
    /// use worth_query_declaration::facade::application_schema::{ApplicationSchemaDeclarationBuilder,
    ///     ApplicationOperationRef, ApplicationDecisionReadScopeRef};
    /// fn missing_grant<S, O, I, Scope>(builder: ApplicationSchemaDeclarationBuilder<S>,
    ///     operation: ApplicationOperationRef<S, O, I>, scope: ApplicationDecisionReadScopeRef<S, Scope>) {
    ///     builder.operation_decision_read_scope(operation, scope);
    /// }
    /// ```
    pub fn operation_decision_read_scope<O, Input, Scope: OperationReads<O>>(
        self,
        operation: ApplicationOperationRef<S, O, Input>,
        scope: ApplicationDecisionReadScopeRef<S, Scope>,
    ) -> Self {
        self.push_member(Member::OperationDecisionReadScope {
            operation: operation.name().to_owned(),
            scope: scope.identity,
        })
    }
}
