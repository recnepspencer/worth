//! Existing keyed revocation owns admission, projection, and the actual status transition.
use super::{grants::*, host::Host, schema::*};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation;
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*,
    declaration::application_schema::*, primary_graph::*,
};
pub fn revoke(
    host: &Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<CapabilitySchema>,
    scope: &WorthQueryRequestScope,
    key: u64,
) -> Result<
    WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCapabilityRevocationDenial<String>,
> {
    let input = RevokeInput {
        identity: DOCUMENT_IDENTITY.to_owned(),
        grant: "source-child".to_owned(),
    };
    host.application
        .runtime()
        .request(principal, scope)
        .execute_capability_revocation_in_program(
            &host.application,
            DocumentPrincipalBinding::reference(),
            RevocationCapability::reference(),
            RevokeGrant::reference(),
            input,
            &key,
            TypedMutationPreconditions::default(),
            |phase, admission| {
                let projected = host
                    .projection
                    .project_admitted_operation(
                        phase,
                        &admission,
                        |reader, document| -> Result<(), String> {
                            let grant = reader
                                .resolve_entity(
                                    GrantIdentity::reference(),
                                    "source-child".to_owned(),
                                )
                                .map_err(|cause| format!("{cause:?}"))?;
                            reader
                                .require_decision_relation(
                                    GrantResource::reference(),
                                    &grant,
                                    document,
                                )
                                .map_err(|cause| format!("{cause:?}"))?;
                            let status = reader
                                .decision_field(&grant, GrantStatus::reference())
                                .map_err(|cause| format!("{cause:?}"))?;
                            if status != Some(1) {
                                return Err("target grant is not active".into());
                            };
                            Ok(())
                        },
                        SystemAllocation,
                    )
                    .map_err(|cause| format!("{cause:?}"))?;
                let (result, projection, _) = projected.into_parts();
                result?;
                let reads = host
                    .application
                    .runtime()
                    .begin_projected_application_read_attempt(
                        phase,
                        admission,
                        projection,
                        SystemAllocation,
                    )
                    .map_err(|cause| format!("{cause:?}"))?;
                let grant = reads
                    .resolve_entity(phase, GrantIdentity::reference(), "source-child".to_owned())
                    .map_err(|cause| format!("{cause:?}"))?;
                reads
                    .complete_projected_dependencies(phase, SystemAllocation)
                    .map_err(|cause| format!("{cause:?}"))?
                    .materialize_capability_revocation_program(&grant)
                    .map_err(|cause| format!("{cause:?}"))
            },
        )
}
