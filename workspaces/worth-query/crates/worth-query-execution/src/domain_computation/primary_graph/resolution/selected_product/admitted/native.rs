//! Original principal admission on a supplied native Query borrow.
use super::*;

impl<'runtime, Schema: ApplicationSchema>
    crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'runtime, Schema>
{
    pub(in crate::domain_computation::primary_graph) fn resolve_authenticated_principal_admitted_with_native_access<
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed_binding: &WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
        validated: Option<&worth_query_installation::facade::WorthQueryValidatedPrincipalBinding>,

        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        WorthQueryPrincipalResolutionDenial,
    >
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        let binding = installed_binding.binding();
        admit_resolution_request(scope, binding, external.is_expired())?;
        let runtime = &self.application().runtime;
        if let Some(validated) = validated {
            if !runtime
                .installed_packages()
                .retains_validated_principal_binding(validated, installed_binding)
            {
                return Err(resolution_denial(
                    WorthQueryPrincipalResolutionDenialKind::Refused(
                        PrincipalRefusal::ForeignRuntime,
                    ),
                    binding,
                ));
            }
        } else {
            runtime
                .installed_packages()
                .validate_principal_binding(installed_binding)
                .map_err(|denial| principal_binding_resolution_denial(denial.kind(), binding))?;
        }
        let (graph, layout) = principal_graph_binding_admitted(runtime, binding)?;
        if graph.binding_identity() != installed_binding.binding_identity()
            || external.binding_identity() != installed_binding.binding_identity()
        {
            return Err(resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::ForeignRuntime),
                binding,
            ));
        }
        let expected_identity = WorthQueryExternalPrincipalIdentityBinding::encode(
            external.identity(),
        )
        .map_err(|_| {
            resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::Refused(
                    PrincipalRefusal::StalePrincipalProof,
                ),
                binding,
            )
        })?;
        let evidence = native.with_runtime_mut(graph, execution, || resolution_denial(
            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::ForeignRuntime), binding,
        ), |relational| {
            let view = relational
                .read_truth()
                .project_snapshot_in_request(self.application_basis().snapshot_handle(), execution)
                .map_err(|cause| entity_lookup_resolution_denial(worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind::Execution(cause), binding))?
                .ok_or_else(|| {
                    resolution_denial(
                        WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::StalePrincipalProof),
                        binding,
                    )
                })?;
            let resolution = WorthQueryPrincipalSnapshotResolution {
                binding,
                layout,
                expected_identity: &expected_identity,
                mode,
                runtime_authority: runtime.authority_identity(),
                binding_identity: graph.binding_identity().clone(),
            };
            let first = crate::domain_computation::primary_graph::resolution::admitted::resolve_at_snapshot(execution, relational, &view, &resolution, installed_binding);
            let result = match first {
                Err(crate::domain_computation::primary_graph::resolution::admitted::AdmittedPrincipalResolutionStop::Index(
                    BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable,
                )) => {
                    ensure_selected_field_indexes_admitted(execution, relational, self.product().relational_basis(), [Some(layout.index_id), None])
                    .map_err(|stop| match stop {
                        SelectedFieldIndexAdmissionStop::Admission(stop) => {
                            principal_admission_denial(stop, binding)
                        }
                        SelectedFieldIndexAdmissionStop::Native => resolution_denial(
                            WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::IdentityIndexUnavailable),
                            binding,
                        ),
                    })?;
                    crate::domain_computation::primary_graph::resolution::admitted::resolve_at_snapshot(execution, relational, &view, &resolution, installed_binding)
                }
                other => other,
            };
            result.map_err(|stop| match stop {

                crate::domain_computation::primary_graph::resolution::admitted::AdmittedPrincipalResolutionStop::Index(kind) => {
                    entity_lookup_resolution_denial(kind, binding)
                }
                crate::domain_computation::primary_graph::resolution::admitted::AdmittedPrincipalResolutionStop::Semantic(kind) => {
                    resolution_denial(kind, binding)
                }
            })
        })?;
        admit_resolution_request(scope, binding, external.is_expired())?;
        Ok(WorthQueryAuthenticatedPrincipal::mint(external, evidence))
    }
}
