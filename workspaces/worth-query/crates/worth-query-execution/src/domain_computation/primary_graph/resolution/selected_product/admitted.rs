//! Fresh principal resolution at the already selected Product/native basis.

use worth_query_installation::facade::WorthQueryPrincipalBindingValidationAdmissionStop;
use worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::RelationalSnapshotProjectionAdmissionStop;

use super::super::*;
use crate::domain_computation::primary_graph::authenticated_principal::PrincipalMintAdmissionStop;
use crate::domain_computation::primary_graph::index_currency::{
    ensure_selected_field_indexes_admitted, SelectedFieldIndexAdmissionStop,
};

/// Principal accepted by the installed decoder on this exact selected root.
/// The selected operation remains borrowed for the whole scoped Query read.
pub(in crate::domain_computation) struct WorthQueryIssuedSelectedPrincipal<
    'selected,
    'runtime,
    Schema,
    Principal,
    PrincipalIdentity,
> {
    principal: WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    selected:
        &'selected crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            'runtime,
            Schema,
        >,
    request: WorthQueryRequestScope,
}

impl<Schema, Principal, PrincipalIdentity>
    WorthQueryIssuedSelectedPrincipal<'_, '_, Schema, Principal, PrincipalIdentity>
{
    pub(in crate::domain_computation) fn principal(
        &self,
    ) -> &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity> {
        &self.principal
    }

    pub(in crate::domain_computation) fn issued_root(
        &self,
    ) -> &worth_relational::facade::branch::AdmittedRelationalBranchBasis {
        self.selected.product().relational_basis()
    }

    pub(in crate::domain_computation) fn issued_snapshot(
        &self,
    ) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.selected.application_basis().snapshot_handle()
    }

    pub(in crate::domain_computation) fn request(&self) -> &WorthQueryRequestScope {
        &self.request
    }

    pub(in crate::domain_computation) fn selected(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'_, Schema>
    {
        self.selected
    }
}

impl<'runtime, Schema> super::super::super::WorthQuerySelectedProductOperation<'runtime, Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation) fn resolve_authenticated_principal_issued<
        'selected,
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &'selected self,
        installed_binding: &WorthQueryInstalledPrincipalBinding<
            Schema,
            Binding,
            Mapping,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        >,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
        validated: Option<&worth_query_installation::facade::WorthQueryValidatedPrincipalBinding>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        WorthQueryIssuedSelectedPrincipal<
            'selected,
            'runtime,
            Schema,
            Principal,
            PrincipalIdentity,
        >,
        WorthQueryPrincipalResolutionDenial,
    >
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        let principal = self.resolve_authenticated_principal_admitted_with_validation(
            installed_binding,
            external,
            request,
            mode,
            validated,
            admission,
        )?;
        admission
            .charge_external_work(1)
            .map_err(|stop| principal_admission_denial(stop, installed_binding.binding()))?;
        Ok(WorthQueryIssuedSelectedPrincipal {
            principal,
            selected: self,
            request: request.clone(),
        })
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_authenticated_principal_admitted<
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &self,
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        WorthQueryPrincipalResolutionDenial,
    >
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        self.resolve_authenticated_principal_admitted_with_validation(
            installed_binding,
            external,
            scope,
            mode,
            None,
            admission,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn resolve_authenticated_principal_admitted_with_validation<
        Binding,
        Mapping,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >(
        &self,
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
        WorthQueryPrincipalResolutionDenial,
    >
    where
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        PrincipalIdentity: 'static,
    {
        let binding = installed_binding.binding();
        // Every terminal Query denial copies this installed name. Reserve its
        // one possible copy before request, installation, or native reads.
        // An admission refusal itself uses an empty subject: copying the
        // name after that refusal would spend unadmitted backing.
        let denial_bytes = u64::try_from(binding.len()).map_err(|_| {
            unfunded_principal_admission_denial(CompanionPreflightStop::WorkCounterOverflow)
        })?;
        let denial_work = denial_bytes.checked_add(1).ok_or_else(|| {
            unfunded_principal_admission_denial(CompanionPreflightStop::WorkCounterOverflow)
        })?;
        admission
            .admit_read_scratch(denial_bytes)
            .and_then(|()| admission.charge_external_work(denial_work))
            .map_err(unfunded_principal_admission_denial)?;
        admit_resolution_request(scope, binding, external.is_expired())?;
        let runtime = &self.application().runtime;
        if let Some(validated) = validated {
            // The receipt's stored and supplied text widths are read by the
            // following checked comparison quote, before that quote can pay.
            admission
                .charge_external_work(6)
                .map_err(|stop| principal_admission_denial(stop, binding))?;
            let compare = validated.comparison_work_bound(binding).ok_or_else(|| {
                principal_admission_denial(CompanionPreflightStop::WorkCounterOverflow, binding)
            })?;
            admission
                .charge_external_work(compare)
                .map_err(|stop| principal_admission_denial(stop, binding))?;
            if !runtime
                .installed_packages()
                .retains_validated_principal_binding(validated, installed_binding)
            {
                return Err(resolution_denial(
                    WorthQueryPrincipalResolutionDenialKind::ForeignRuntime,
                    binding,
                ));
            }
        } else {
            runtime
                .installed_packages()
                .validate_principal_binding_admitted(installed_binding, |work, bytes| {
                    admission.admit_read_scratch(bytes)?;
                    admission.charge_external_work(work)
                })
                .map_err(|stop| match stop {
                    WorthQueryPrincipalBindingValidationAdmissionStop::Admission(stop) => {
                        principal_admission_denial(stop, binding)
                    }
                    WorthQueryPrincipalBindingValidationAdmissionStop::Validation(denial) => {
                        principal_binding_resolution_denial(denial.kind(), binding)
                    }
                    WorthQueryPrincipalBindingValidationAdmissionStop::AccountingOverflow => {
                        principal_admission_denial(
                            CompanionPreflightStop::WorkCounterOverflow,
                            binding,
                        )
                    }
                })?;
        }
        let (graph, layout) = principal_graph_binding_admitted(runtime, binding, admission)?;
        if graph.binding_identity() != installed_binding.binding_identity()
            || external.binding_identity() != installed_binding.binding_identity()
        {
            return Err(resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::ForeignRuntime,
                binding,
            ));
        }
        let (copy_bytes, copy_work) = external.identity().index_encoding_copy_requirements();
        let copy_bytes = u64::try_from(copy_bytes).map_err(|_| {
            principal_admission_denial(
                CompanionPreflightStop::PreparationMemoryCounterOverflow,
                binding,
            )
        })?;
        let copy_work = u64::try_from(copy_work)
            .ok()
            .and_then(|work| work.checked_add(2))
            .ok_or_else(|| {
                principal_admission_denial(CompanionPreflightStop::WorkCounterOverflow, binding)
            })?;
        admission
            .admit_read_scratch(copy_bytes)
            .and_then(|()| admission.charge_external_work(copy_work))
            .map_err(|stop| principal_admission_denial(stop, binding))?;
        let expected_identity = WorthQueryExternalPrincipalIdentityBinding::encode(
            external.identity(),
        )
        .map_err(|_| {
            resolution_denial(
                WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof,
                binding,
            )
        })?;
        let evidence = graph.with_runtime_mut(|relational| {
            let view = relational
                .read_truth()
                .project_snapshot_admitted(
                    self.application_basis().snapshot_handle(),
                    |work, bytes| {
                        admission.admit_read_scratch(bytes)?;
                        admission.charge_external_work(work)
                    },
                )
                .map_err(|stop| match stop {
                    RelationalSnapshotProjectionAdmissionStop::Admission(stop) => {
                        principal_admission_denial(stop, binding)
                    }
                    RelationalSnapshotProjectionAdmissionStop::AccountingOverflow => {
                        principal_admission_denial(
                            CompanionPreflightStop::WorkCounterOverflow,
                            binding,
                        )
                    }
                })?
                .ok_or_else(|| {
                    resolution_denial(
                        WorthQueryPrincipalResolutionDenialKind::StalePrincipalProof,
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
            let first = super::super::admitted::resolve_at_snapshot(
                relational,
                &view,
                &resolution,
                installed_binding,
                admission,
            );
            let result = match first {
                Err(super::super::admitted::AdmittedPrincipalResolutionStop::Index(
                    BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable,
                )) => {
                    ensure_selected_field_indexes_admitted(
                        relational,
                        self.product().relational_basis(),
                        [Some(layout.index_id), None],
                        admission,
                    )
                    .map_err(|stop| match stop {
                        SelectedFieldIndexAdmissionStop::Admission(stop) => {
                            principal_admission_denial(stop, binding)
                        }
                        SelectedFieldIndexAdmissionStop::Native => resolution_denial(
                            WorthQueryPrincipalResolutionDenialKind::IdentityIndexUnavailable,
                            binding,
                        ),
                    })?;
                    super::super::admitted::resolve_at_snapshot(
                        relational,
                        &view,
                        &resolution,
                        installed_binding,
                        admission,
                    )
                }
                other => other,
            };
            result.map_err(|stop| match stop {
                super::super::admitted::AdmittedPrincipalResolutionStop::Admission(stop) => {
                    principal_admission_denial(stop, binding)
                }
                super::super::admitted::AdmittedPrincipalResolutionStop::Index(kind) => {
                    entity_lookup_resolution_denial(kind, binding)
                }
                super::super::admitted::AdmittedPrincipalResolutionStop::Semantic(kind) => {
                    resolution_denial(kind, binding)
                }
            })
        })?;
        admit_resolution_request(scope, binding, external.is_expired())?;
        WorthQueryAuthenticatedPrincipal::mint_admitted(external, evidence, |work, bytes| {
            admission.admit_read_scratch(bytes)?;
            admission.charge_external_work(work)
        })
        .map_err(|stop| match stop {
            PrincipalMintAdmissionStop::Admission(stop) => {
                principal_admission_denial(stop, binding)
            }
            PrincipalMintAdmissionStop::AccountingOverflow => {
                principal_admission_denial(CompanionPreflightStop::WorkCounterOverflow, binding)
            }
        })
    }
}

fn unfunded_principal_admission_denial(
    stop: CompanionPreflightStop,
) -> WorthQueryPrincipalResolutionDenial {
    let kind = match stop {
        CompanionPreflightStop::PreparationMemoryExhausted { .. }
        | CompanionPreflightStop::PreparationMemoryCounterOverflow => {
            WorthQueryPrincipalResolutionDenialKind::ProjectionPreparationMemoryExhausted
        }
        _ => WorthQueryPrincipalResolutionDenialKind::ProjectionWorkBudgetExceeded,
    };
    WorthQueryPrincipalResolutionDenial::new(kind, String::new())
}
