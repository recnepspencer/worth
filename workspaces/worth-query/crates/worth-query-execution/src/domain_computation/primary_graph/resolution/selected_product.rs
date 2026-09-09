use super::*;

impl<Schema> super::super::WorthQuerySelectedProductOperation<'_, Schema>
where
    Schema: ApplicationSchema,
{
    pub fn resolve_authenticated_principal<Binding, Mapping, Principal, PrincipalIdentity>(
        &self,
        installed_binding: &WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
        >,
        external: WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<
        WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        WorthQueryPrincipalResolutionDenial,
    >
    where
        PrincipalIdentity: TypedApplicationIdentityValue,
    {
        admit_resolution_request(scope, installed_binding.binding(), external.is_expired())?;
        let runtime = &self.application().runtime;
        runtime
            .installed_packages()
            .validate_principal_binding(installed_binding)
            .map_err(|denial| {
                principal_binding_resolution_denial(denial.kind(), installed_binding.binding())
            })?;
        let (graph, layout) = principal_graph_binding(runtime, installed_binding.binding())?;
        if graph.binding_identity() != installed_binding.binding_identity()
            || external.binding_identity() != installed_binding.binding_identity()
        {
            return Err(resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::ForeignRuntime,
                installed_binding.binding(),
            ));
        }
        let expected_identity = external.identity().clone().into_foundational_value();
        let handle = graph.integration_handle();
        let evidence = handle.with_runtime_mut(|relational| {
            handle
                .ensure_primary_indexes_for_basis(relational, self.product().relational_basis())
                .map_err(|_| {
                    resolution_denial(
                        WorthQueryPrincipalResolutionDenialKind::IdentityIndexUnavailable,
                        installed_binding.binding(),
                    )
                })?;
            resolve_at_snapshot::<PrincipalIdentity>(
                relational,
                self.application_basis().snapshot_handle(),
                &WorthQueryPrincipalSnapshotResolution {
                    binding: installed_binding.binding(),
                    layout: &layout,
                    expected_identity: &expected_identity,
                    mode,
                    runtime_authority: runtime.authority_identity(),
                    binding_identity: graph.binding_identity().clone(),
                },
            )
        })?;
        admit_resolution_request(scope, installed_binding.binding(), external.is_expired())?;
        Ok(WorthQueryAuthenticatedPrincipal::mint(external, evidence))
    }

    pub fn validate_authenticated_principal<Principal, PrincipalIdentity>(
        &self,
        principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        scope: &WorthQueryRequestScope,
    ) -> Result<(), WorthQueryPrincipalResolutionDenial> {
        admit_resolution_request(scope, principal.binding(), principal.is_expired())?;
        let runtime = &self.application().runtime;
        if principal.runtime_authority() != runtime.authority_identity() {
            return Err(resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::ForeignRuntime,
                principal.binding(),
            ));
        }
        let (graph, layout) = principal_graph_binding(runtime, principal.binding())?;
        validate_current_schema_binding::<Schema>(
            runtime,
            graph.binding_identity(),
            principal.binding_identity(),
            principal.binding(),
        )?;
        let expected_identity = principal
            .external_identity()
            .clone()
            .into_foundational_value();
        let handle = graph.integration_handle();
        handle.with_runtime_mut(|relational| {
            handle
                .ensure_primary_indexes_for_basis(relational, self.product().relational_basis())
                .map_err(|_| {
                    resolution_denial(
                        WorthQueryPrincipalResolutionDenialKind::IdentityIndexUnavailable,
                        principal.binding(),
                    )
                })?;
            validate_freshness_at_snapshot(
                relational,
                self.application_basis().snapshot_handle(),
                principal,
                &layout,
                &expected_identity,
            )
        })?;
        admit_resolution_request(scope, principal.binding(), principal.is_expired())
    }
}
