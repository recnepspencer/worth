use std::marker::PhantomData;
use std::sync::Arc;

use crate::domain_computation::primary_graph::application_attempt::read_phase::WorthQueryOrdinaryApplicationRead;
use crate::domain_computation::primary_graph::application_attempt::read_scope::WorthQueryApplicationReadScope;
use crate::domain_computation::primary_graph::application_attempt::read_set::source_facts::SourceFacts;
use crate::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenial;
use crate::domain_computation::primary_graph::application_contribution::CompletedComputationRetention;
use crate::domain_computation::primary_graph::invariant_projection::{
    ConsumedOutputEvidence, ExpectedFactKeys, ObservedComputationInputs,
};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::value_decode::ValueDecodeState;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryInstalledEntityResolutionContext,
};

/// Unexported crate-boundary initialization bundle for admitting a
/// `WorthQueryApplicationReadAttempt` from `worth-query-execution-query`.
pub struct WorthQueryApplicationReadAttemptInit<
    'request,
    'authority,
    Schema,
    Operation,
    Input,
    Scope,
    Phase = WorthQueryOrdinaryApplicationRead,
> {
    pub execution: worth_execution::ExecutionRequest<'request, 'authority>,
    pub value_decode: ValueDecodeState,
    pub admission: WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    pub lease: WorthQueryApplicationSnapshotLease,
    pub layout: Arc<WorthQueryPrimaryGraphLayout>,
    pub entity_resolution: WorthQueryInstalledEntityResolutionContext,
    pub read_scope: WorthQueryApplicationReadScope,
    pub expected_facts: Option<ExpectedFactKeys>,
    pub computation_reads: (ObservedComputationInputs, CompletedComputationRetention),
    pub source_facts: SourceFacts,
    pub consumed_outputs: Vec<ConsumedOutputEvidence>,
    pub _phase: PhantomData<fn() -> Phase>,
}

/// Crate-boundary port for admitting a `WorthQueryApplicationReadAttempt`.
/// Exported only on `worth_query_execution_attempt::facade::ports`, never on
/// `worth_query_execution::facade` or Host.
pub trait ApplicationReadAttemptAdmissionPort<
    'request,
    'authority,
    Schema,
    Operation,
    Input,
    Scope,
    Phase,
>: Sized
{
    fn admit_read_attempt(
        init: WorthQueryApplicationReadAttemptInit<
            'request,
            'authority,
            Schema,
            Operation,
            Input,
            Scope,
            Phase,
        >,
    ) -> Result<Self, WorthQueryApplicationAttemptDenial>;
}
