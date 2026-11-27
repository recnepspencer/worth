//! Resolve an unauthenticated candidate identity through the installed
//! principal binding, exactly as request admission resolves a proven one.
//!
//! A candidate carries no proof: nothing here can open a door. The result is
//! the principal entity an eligibility observation evaluates, or the typed
//! reason admission would refuse this identity before any policy is read.
use super::*;
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

/// Why admission would refuse a candidate identity before reading policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCandidatePrincipalRefusal {
    /// No mapping carries this identity, or it links no principal.
    Unmapped,
    /// The identity's mapping is disabled.
    Disabled,
    /// More than one mapping, or more than one principal, answers the identity.
    Ambiguous,
}

impl<'runtime, Schema: ApplicationSchema>
    crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'runtime, Schema>
{
    /// The principal entity `identity` resolves to at this selected basis.
    pub fn resolve_candidate_principal_with_native_access<
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
        identity: &WorthQueryExternalPrincipalIdentity,
        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<
        Result<worth_relational::facade::identity::EntityId, WorthQueryCandidatePrincipalRefusal>,
        WorthQueryPrincipalResolutionDenial,
    >
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        let binding = installed_binding.binding();
        let runtime = self.application().runtime.execution();
        runtime
            .installed_packages()
            .validate_principal_binding(installed_binding)
            .map_err(|denial| principal_binding_resolution_denial(denial.kind(), binding))?;
        let (graph, layout) = principal_graph_binding_admitted(runtime, binding)?;
        if graph.binding_identity() != installed_binding.binding_identity() {
            return Err(resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::ForeignRuntime),
                binding,
            ));
        }
        let expected_identity = WorthQueryExternalPrincipalIdentityBinding::encode(identity)
            .map_err(|_| {
                resolution_denial(
                    WorthQueryPrincipalResolutionDenialKind::Refused(
                        PrincipalRefusal::StalePrincipalProof,
                    ),
                    binding,
                )
            })?;
        let foreign = || {
            resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::Refused(PrincipalRefusal::ForeignRuntime),
                binding,
            )
        };
        native.with_runtime(graph, execution, foreign, |relational| {
            let view = self.project_candidate_view(execution, relational, binding)?;
            let resolution = WorthQueryPrincipalSnapshotResolution {
                binding,
                layout,
                expected_identity: &expected_identity,
                mode: WorthQueryPrincipalResolutionMode::Ordinary,
                runtime_authority: runtime.authority_identity(),
                binding_identity: graph.binding_identity().clone(),
            };
            let resolve = || {
                crate::domain_computation::primary_graph::resolution::admitted::resolve_at_snapshot(
                    execution,
                    relational,
                    &view,
                    &resolution,
                    installed_binding,
                )
            };
            let resolved = match resolve() {
                Err(AdmittedStop::Index(
                    BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable,
                )) => {
                    self.admit_identity_index(execution, relational, layout.index_id, binding)?;
                    resolve()
                }
                other => other,
            };
            classify_candidate(resolved, binding)
        })
    }

    /// The selected snapshot's truth, projected under this request.
    fn project_candidate_view(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        binding: &str,
    ) -> Result<
        worth_relational::facade::runtime::VisibilityProjectionView,
        WorthQueryPrincipalResolutionDenial,
    > {
        relational
            .read_truth()
            .project_snapshot_in_request(self.application_basis().snapshot_handle(), execution)
            .map_err(|cause| {
                entity_lookup_resolution_denial(
                    BoundedEntityFieldLookupDenialKind::Execution(cause),
                    binding,
                )
            })?
            .ok_or_else(|| {
                resolution_denial(
                    WorthQueryPrincipalResolutionDenialKind::Refused(
                        PrincipalRefusal::StalePrincipalProof,
                    ),
                    binding,
                )
            })
    }

    /// Brings the identity index current at the selected root, as admission
    /// does before retrying an exact-generation lookup.
    fn admit_identity_index(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        index: worth_relational::facade::indexes::DerivedIndexId,
        binding: &str,
    ) -> Result<(), WorthQueryPrincipalResolutionDenial> {
        ensure_selected_field_indexes_admitted(
            execution,
            relational,
            self.product().relational_basis(),
            [Some(index), None],
        )
        .map_err(|stop| match stop {
            SelectedFieldIndexAdmissionStop::Admission(stop) => {
                principal_admission_denial(stop, binding)
            }
            SelectedFieldIndexAdmissionStop::Native => resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::Refused(
                    PrincipalRefusal::IdentityIndexUnavailable,
                ),
                binding,
            ),
        })
    }
}

type AdmittedStop =
    crate::domain_computation::primary_graph::resolution::admitted::AdmittedPrincipalResolutionStop;

impl WorthQueryCandidatePrincipalRefusal {
    /// The refusal admission reports for an identity that resolves to no
    /// single enabled principal; `None` for every other resolution stop.
    pub const fn of(kind: WorthQueryPrincipalResolutionDenialKind) -> Option<Self> {
        use WorthQueryPrincipalResolutionDenialKind::Refused;
        match kind {
            Refused(
                PrincipalRefusal::UnknownPrincipal | PrincipalRefusal::MissingPrincipalTarget,
            ) => Some(Self::Unmapped),
            Refused(PrincipalRefusal::DisabledPrincipal) => Some(Self::Disabled),
            Refused(
                PrincipalRefusal::AmbiguousPrincipal | PrincipalRefusal::AmbiguousPrincipalTarget,
            ) => Some(Self::Ambiguous),
            _ => None,
        }
    }
}

/// Admission's own refusals of an unproven identity become the candidate's
/// verdict; every other stop still refuses the observation itself.
fn classify_candidate<PrincipalIdentity>(
    resolved: Result<WorthQueryResolvedPrincipalEvidence<PrincipalIdentity>, AdmittedStop>,
    binding: &str,
) -> Result<
    Result<worth_relational::facade::identity::EntityId, WorthQueryCandidatePrincipalRefusal>,
    WorthQueryPrincipalResolutionDenial,
> {
    match resolved {
        Ok(evidence) => Ok(Ok(evidence.principal_entity_id)),
        Err(AdmittedStop::Semantic(kind)) => match WorthQueryCandidatePrincipalRefusal::of(kind) {
            Some(refusal) => Ok(Err(refusal)),
            None => Err(resolution_denial(kind, binding)),
        },
        Err(AdmittedStop::Index(kind)) => Err(entity_lookup_resolution_denial(kind, binding)),
    }
}
