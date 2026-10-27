//! Both recovery inspection doors consume real current admission evidence.
use super::declaration::{MonitorInput, MonitorInspectionCapability, MonitorOperation};
use crate::document_retention_model::schema::*;
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::{admission::authenticated_principal::*, primary_graph::*};

pub fn inspect(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<DocumentRetentionSchema>,
    handle: &WorthQueryRecoveryHandle,
    actor: &WorthQueryAuthenticatedExternalPrincipal<DocumentRetentionSchema>,
    request: &WorthQueryRequestScope,
    original: &MonitorInput,
) -> WorthQueryRecoveryInspectAuthority {
    runtime
        .with_application_advancement(request, |phase| {
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            let installed = runtime.installed_schema();
            let principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    &installed
                        .principal_binding(DocumentPrincipalBinding::reference())
                        .unwrap(),
                    actor,
                    request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let access = selected
                .admit_capability_access(
                    &phase,
                    &principal,
                    &installed
                        .capability(
                            MonitorInspectionCapability::reference(),
                            MonitorOperation::reference(),
                        )
                        .unwrap(),
                    original.clone(),
                    request,
                )
                .unwrap();
            let disclosure = runtime
                .admit_recovery_inspection_disclosure(&access)
                .unwrap();
            let operation = runtime
                .authorize_capability_operation(
                    &phase,
                    access,
                    &installed
                        .installed_operation(MonitorOperation::reference())
                        .unwrap(),
                    TypedMutationPreconditions::default(),
                )
                .unwrap();
            runtime
                .admit_recovery_inspect_authority(handle, &operation, &disclosure)
                .unwrap()
        })
        .unwrap()
}
