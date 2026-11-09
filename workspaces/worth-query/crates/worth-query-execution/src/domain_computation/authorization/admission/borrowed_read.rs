//! Independent operation admission through the caller's original Native loan.
use super::validation::{denial, validate_admission_clock, validate_static_authority_prefix};
use crate::domain_computation::{
    authorization::{
        WorthQueryOperationAuthorizationDenial as Denial,
        WorthQueryOperationAuthorizationDenialKind as Kind,
        WorthQueryOperationAuthorizationRefusalKind as Refusal,
        WorthQueryRetainedAuthorizationDecisionFacts,
    },
    primary_graph::{
        WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
        WorthQueryPrimaryGraphApplicationRuntime,
    },
    provider_session::WorthQueryGraphWorkSessionIdentity,
};
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::{ApplicationSchema, InstalledOperationView};
use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};

type Observe<'a> = dyn Fn(
        &worth_execution::ExecutionRequest<'_, '_>,
        &RelationalRuntime,
        SnapshotHandle,
        InstalledOperationView<'_>,
    ) -> Result<WorthQueryRetainedAuthorizationDecisionFacts, Denial>
    + 'a;

/// This owner alone constructs the callback. It holds genuine issued C inputs,
/// not the outer operation's grant, and cannot acquire a Native transport.
pub(in crate::domain_computation) struct BorrowedReadOperationAdmission<'a> {
    observe: Box<Observe<'a>>,
    pub(in crate::domain_computation) session: WorthQueryGraphWorkSessionIdentity,
    pub(in crate::domain_computation) principal: worth_relational::facade::identity::EntityId,
    pub(in crate::domain_computation) scope: worth_relational::facade::identity::EntityId,
}
impl<'a> BorrowedReadOperationAdmission<'a> {
    pub(in crate::domain_computation) fn new<S, P, I, E>(
        runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<S>,
        principal: &'a WorthQueryAuthenticatedPrincipal<S, P, I>,
        scope: &'a WorthQueryApplicationEntityIdentity<S, E>,
        request: &'a WorthQueryRequestScope,
        session: WorthQueryGraphWorkSessionIdentity,
    ) -> Self
    where
        S: ApplicationSchema,
    {
        Self {
            session,
            principal: principal.principal_entity_id(),
            scope: scope.entity_id(),
            observe: Box::new(move |execution, native, snapshot, operation| {
                let result = (|| {
                    validate_admission_clock(
                        &runtime.runtime,
                        principal.clock(),
                        request,
                        operation.operation(),
                    )?;
                    super::validation::admit_request(request, operation.operation())?;
                    validate_static_authority_prefix(runtime, principal, scope, operation)?;
                    runtime
                        .observe_current_operation_borrowed(
                            execution, principal, scope, operation, session, native, snapshot,
                        )?
                        .retain_for(operation)
                })();
                result.map_err(|cause: Denial| {
                    denial(
                        cause.kind(),
                        format!("{}: {}", operation.operation(), cause.subject()),
                    )
                })
            }),
        }
    }
    pub(in crate::domain_computation) fn observe(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
        snapshot: SnapshotHandle,
        operation: InstalledOperationView<'_>,
    ) -> Result<WorthQueryRetainedAuthorizationDecisionFacts, Denial> {
        // A read cannot inherit an outer producer's capability. Its own
        // conventional requirements must authorize this original principal/scope.
        if operation.contracts().authorization() == worth_query_installation::facade::WorthQueryInstalledApplicationOperationAuthorization::Capability {
            return Err(denial(Kind::Refused(Refusal::CapabilityAuthorizationMissing), operation.operation()));
        }
        (self.observe)(execution, native, snapshot, operation)
    }
}
