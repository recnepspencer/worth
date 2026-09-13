use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryHostConditionalPredicateProvider,
    WorthQueryInstalledTemporalConditionalOperation, WorthQueryNamedClock,
    WorthQueryNamedClockSource, WorthQueryPortableApplicationConditionalOperationBinding,
    WorthQueryTemporalIntentProjector,
};

use super::{
    ApplicationConditionalBindingScope, WorthQueryConditionalApplicationRuntimeInstallation,
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind as DenialKind,
};

impl<Schema> WorthQueryConditionalApplicationRuntimeInstallation<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn installed_schema(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<Schema> {
        &self.publication.installed_schema
    }

    pub fn installed_packages(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledPackageIndex {
        self.publication.runtime.installed_packages()
    }

    pub fn retain_invariant_projection_authority(
        &self,
    ) -> super::super::super::WorthQueryApplicationInvariantProjectionAuthority<Schema> {
        self.publication
            .bootstrap
            .retain_invariant_projection_authority()
    }

    pub(in crate::domain_computation::primary_graph) fn begin_application_binding_scope(
        &mut self,
        binding: WorthQueryPortableApplicationConditionalOperationBinding,
        node_identity: String,
    ) {
        assert!(
            self.application_binding_scope.is_none(),
            "application conditional binding scopes cannot nest"
        );
        self.application_binding_scope = Some(ApplicationConditionalBindingScope {
            binding,
            node_identity,
            initial_binding_count: self.bindings.len(),
        });
    }

    pub(in crate::domain_computation::primary_graph) fn finish_application_binding_scope(
        &mut self,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
        let scope = self
            .application_binding_scope
            .take()
            .expect("an application conditional binding scope was begun");
        if self.bindings.len() == scope.initial_binding_count.saturating_add(1) {
            Ok(())
        } else {
            Err(denial(
                DenialKind::IncompleteBindingInventory,
                scope.node_identity,
            ))
        }
    }

    pub(in crate::domain_computation::primary_graph) fn abandon_application_binding_scope(
        &mut self,
    ) {
        self.application_binding_scope = None;
    }

    pub(super) fn validate_application_binding_scope<
        ApplicationOperation,
        Input,
        D,
        O,
        F,
        Node,
        Provider,
        Clock,
        Source,
        Query,
        Parameters,
        QueryResult,
        Scope,
        Projector,
    >(
        &self,
        binding: &WorthQueryInstalledTemporalConditionalOperation<
            Schema,
            ApplicationOperation,
            Input,
            D,
            O,
            F,
            Node,
            Provider,
            Clock,
            Source,
            Query,
            Parameters,
            QueryResult,
            Scope,
            Projector,
        >,
    ) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial>
    where
        Provider: WorthQueryHostConditionalPredicateProvider<Node>,
        Clock: WorthQueryNamedClock,
        Source: WorthQueryNamedClockSource<Clock>,
        Projector: WorthQueryTemporalIntentProjector<Node, Clock, QueryResult, Input>,
    {
        let Some(scope) = &self.application_binding_scope else {
            return Ok(());
        };
        let node = binding.clocked_node().provider().node();
        if !scope_matches(
            scope,
            node.operation().binding(),
            node.location().node_identity(),
        ) {
            return Err(denial(
                DenialKind::ForeignBinding,
                scope.node_identity.clone(),
            ));
        }
        Ok(())
    }
}

fn scope_matches(
    scope: &ApplicationConditionalBindingScope,
    binding: &WorthQueryPortableApplicationConditionalOperationBinding,
    node_identity: &str,
) -> bool {
    binding == &scope.binding && node_identity == scope.node_identity
}

fn denial(
    kind: DenialKind,
    subject: impl Into<String>,
) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryConditionalRuntimeInstallationDenial::new(kind, subject)
}

#[cfg(test)]
mod tests {
    use worth_query_installation::facade::{
        WorthQueryPortableApplicationConditionalOperationBinding,
        WorthQueryPortableApplicationConditionalOperationBindingParts,
    };

    use super::{scope_matches, ApplicationConditionalBindingScope};

    fn binding(operation: &str) -> WorthQueryPortableApplicationConditionalOperationBinding {
        WorthQueryPortableApplicationConditionalOperationBinding::from_untrusted_parts(
            WorthQueryPortableApplicationConditionalOperationBindingParts {
                schema_owner: "owner".into(),
                schema_name: "Schema".into(),
                application_operation: operation.into(),
                input_type: worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity::from_untrusted("input".into()),
                domain_operation_slot: "domain".into(),
                domain_operation_canonical_identity: "canonical".into(),
            },
        )
    }

    #[test]
    fn application_binding_scope_rejects_wrong_operation_and_node() {
        let scope = ApplicationConditionalBindingScope {
            binding: binding("expected"),
            node_identity: "ready".into(),
            initial_binding_count: 0,
        };
        assert!(scope_matches(&scope, &binding("expected"), "ready"));
        assert!(!scope_matches(&scope, &binding("foreign"), "ready"));
        assert!(!scope_matches(&scope, &binding("expected"), "wrong"));
    }
}
