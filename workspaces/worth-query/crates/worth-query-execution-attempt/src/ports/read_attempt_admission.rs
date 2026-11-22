use std::sync::Arc;

use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity, InstalledOperationCatalog,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use worth_runtime_world::facade::RuntimeWorldOwnerIdentity;

use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::application_attempt::read_phase::{
    WorthQueryOrdinaryApplicationRead, WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::application_attempt::read_set::WorthQueryApplicationReadAttempt;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial;
use crate::domain_computation::primary_graph::invariant_projection::{
    AppliedOperationProjection, WorthQueryApplicationOperationInvariantProjectionSnapshot,
};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::value_decode::ValueDecodeState;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryInstalledEntityResolutionContext,
};

/// Installed application runtime context supplied to `worth-query-execution-attempt`
/// when admitting a read attempt. Carries no caller-assembled lease, phase,
/// read scope, or expected facts; those are verified and derived inside `attempt`.
pub struct WorthQueryReadAttemptRuntimeContext<'a> {
    pub world_owner: RuntimeWorldOwnerIdentity,
    pub runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    pub binding_identity: ApplicationSchemaBindingIdentity,
    pub installed_operations: &'a InstalledOperationCatalog,
    pub primary_graph: Option<(
        Arc<WorthQueryPrimaryGraphLayout>,
        WorthQueryInstalledEntityResolutionContext,
    )>,
    pub value_decode: ValueDecodeState,
}

/// Crate-boundary port for admitting a `WorthQueryApplicationReadAttempt`.
/// Exported only on `worth_query_execution_attempt::facade::ports`, never on
/// `worth_query_execution::facade` or Host. Every transition validates home,
/// application, installed-operation, current-authority, and projection/lease
/// affinity inside `worth-query-execution-attempt`.
pub trait ApplicationReadAttemptAdmissionPort<
    'request,
    'authority,
    Schema: ApplicationSchema,
    Operation,
    Input,
    Scope,
>: Sized
{
    fn admit_ordinary_read_attempt(
        execution: &ExecutionRequest<'request, 'authority>,
        context: WorthQueryReadAttemptRuntimeContext<'_>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'request,
            'authority,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryOrdinaryApplicationRead,
        >,
        WorthQueryApplicationAttemptDenial,
    >;

    fn admit_projected_snapshot_read_attempt(
        execution: &ExecutionRequest<'request, 'authority>,
        context: WorthQueryReadAttemptRuntimeContext<'_>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'request,
            'authority,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    >;

    fn admit_applied_projection_read_attempt(
        execution: &ExecutionRequest<'request, 'authority>,
        context: WorthQueryReadAttemptRuntimeContext<'_>,
        admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: AppliedOperationProjection<Schema, Operation>,
        candidate: RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
    ) -> Result<
        WorthQueryApplicationReadAttempt<
            'request,
            'authority,
            Schema,
            Operation,
            Input,
            Scope,
            WorthQueryProjectedApplicationMutation,
        >,
        WorthQueryApplicationAttemptDenial,
    >;
}
