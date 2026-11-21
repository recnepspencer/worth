use crate::equality_type_id::EqualityTypeId as TypeId;
use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::Arc;

use worth_query_installation::facade::{ApplicationReadableScalarValueBinding, ApplicationSchema};

use super::read_basis::NativeProjectionRead;
use super::work::WorthQueryInvariantProjectionWorkBudget;
use super::{
    WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryApplicationInvariantProjectionSnapshot, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionDenial, WorthQueryInvariantProjectionWork,
    WorthQueryRealizedProjectionScope,
};
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial;

pub mod borrowed_snapshot;
pub use borrowed_snapshot::NativeReadAccess;
#[path = "locked_reader/content.rs"]
pub mod content;
#[path = "locked_reader/operation_body.rs"]
pub mod operation_body;
pub mod predecessor;
#[path = "locked_reader/predecode_admission.rs"]
pub mod predecode_admission;
pub mod reader_execution;
pub use borrowed_snapshot::CompletedProjectionRead;
pub mod requirement_collection;
pub use requirement_collection::ObservedRequirementItems;
#[path = "locked_reader/traversal_denial.rs"]
pub mod traversal_denial;
pub use traversal_denial::{
    WorthQueryInvariantProjectionTraversalDenial, WorthQueryInvariantProjectionTraversalDenialKind,
};

/// The reader a projection closure uses to read the main branch at one
/// snapshot.
///
/// It resolves entities by identity field, reads fields, traverses relations,
/// and summarizes aggregates. Every read is charged to the projection's work.
pub struct WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
    pub value_bindings: &'runtime worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    pub value_decode: &'runtime super::super::value_decode::ValueDecodeSession<'runtime>,
    pub execution: &'runtime worth_execution::ExecutionRequest<'runtime, 'runtime>,
    pub native: NativeProjectionRead<'runtime>,
    pub layout: &'runtime super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    pub snapshot: &'runtime worth_relational::facade::snapshots::SnapshotHandle,
    pub entity_resolution: &'runtime super::super::WorthQueryInstalledEntityResolutionContext,
    pub observation: worth_relational::facade::mvcc::RelationalBranchObservation,
    pub authority_identity: u64,
    pub binding_identity: &'runtime worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    pub program_activation:
        Option<&'runtime super::super::program_occurrence::WorthQueryProgramActivationCell>,
    pub work: WorthQueryInvariantProjectionWork,
    pub work_budget: WorthQueryInvariantProjectionWorkBudget,
    /// A read found no capacity to retain its evidence. The projection then
    /// ends as that capacity's stop, whatever the closure made of the read's
    /// refusal, as it ends for an exceeded work budget.
    pub retention_exhausted: bool,
    /// The first native read refusal vetoes the closure's output at completion.
    pub native_read_stop:
        Option<worth_relational::facade::transactions::RelationalExecutionDenialCause>,
    pub realized_scope: WorthQueryRealizedProjectionScope,
    pub aggregate_projections:
        Arc<std::sync::Mutex<super::super::aggregate_projection::WorthQueryAggregateProjections>>,
    pub output_lineage:
        Arc<std::sync::Mutex<super::super::output_lineage::WorthQueryApplicationOutputLineage>>,
    pub invalidation_owner: Arc<super::super::SourceInvalidationOwner>,
    pub managed_views: Arc<dyn crate::ports::ManagedDerivedViewPort>,
    pub selected_product_occurrence:
        Option<worth_runtime_world::facade::ProductBranchIncarnation>,
    pub selected_product_generation: Option<u64>,
    pub selected_source_partition_identity: Option<[u8; 32]>,
    pub prior_output_bindings:
        HashMap<TypeId, Arc<super::super::WorthQueryApplicationOutputCorrespondence>>,
    pub current_output_families: HashMap<
        (TypeId, worth_relational::facade::identity::EntityId),
        Vec<super::operation_reader::CertifiedOutputCorrespondence>,
    >,
    pub dependent_source_facts:
        super::super::application_attempt::retained_decision_facts::AuthoringSourceFacts,
    pub retention_control:
        super::super::application_attempt::retained_decision_facts::StorageControl<
            'runtime,
            'runtime,
        >,
    pub applied_consumptions: Vec<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>,
    pub consumed_outputs: BTreeMap<
        Arc<super::super::output_lineage::RecordedSettlementIdentity>,
        super::ConsumedOutputEvidence,
    >,
    _schema: PhantomData<fn() -> Schema>,
}

/// The result of an invariant projection: the closure's output, the work it
/// spent, and the snapshot it read.
///
/// `into_parts` hands back the snapshot for further reads at the same version.
pub struct WorthQueryCompletedInvariantProjection<Schema, Output> {
    output: Output,
    snapshot: WorthQueryApplicationInvariantProjectionSnapshot<Schema>,
    work: WorthQueryInvariantProjectionWork,
}

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn project<Output>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompletedInvariantProjection<Schema, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        let phase_request = phase
            .request_for_source(self.graph.relational_runtime_instance_id())
            .map_err(|_| {
                WorthQueryInvariantProjectionDenial::from_kind(
                    super::WorthQueryInvariantProjectionDenialKind::ForeignPhase,
                )
            })?;
        let execution = &phase_request;

        let basis = self.graph.with_runtime(execution, |runtime| {
            let identity = runtime.main_branch_identity();
            runtime
                .observe_branch(&identity)
                .map(|(_, basis)| basis)
                .map_err(super::admission_denial::from_branch_basis_denial)
        })?;
        self.project_with_work_budget(
            execution,
            WorthQueryInvariantProjectionWorkBudget::unbounded(),
            basis,
            projection,
            super::super::application_attempt::retained_decision_facts::StorageControl::new(
                allocation_policy,
                None,
            ),
        )
    }

    pub fn project_with_work_limit<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        maximum_work: Option<usize>,
        basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompletedInvariantProjection<Schema, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        self.project_with_work_budget(
            execution,
            maximum_work.map_or_else(
                WorthQueryInvariantProjectionWorkBudget::unbounded,
                WorthQueryInvariantProjectionWorkBudget::bounded,
            ),
            basis,
            projection,
            super::super::application_attempt::retained_decision_facts::StorageControl::new(
                allocation_policy,
                request,
            )
            .with_execution(*execution),
        )
    }

    fn project_with_work_budget<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        work_budget: WorthQueryInvariantProjectionWorkBudget,
        basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        retention_control: super::super::application_attempt::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<
        WorthQueryCompletedInvariantProjection<Schema, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        let snapshot = self.graph.with_runtime(execution, |runtime| {
            runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .map_err(super::admission_denial::from_snapshot_admission_denial)
        })?;
        let projected = catch_unwind(AssertUnwindSafe(|| {
            self.read_snapshot(
                execution,
                work_budget,
                &basis,
                &snapshot,
                projection,
                retention_control,
            )
        }));
        let CompletedProjectionRead {
            output,
            work,
            realized_scope,
            consumed_outputs,
            applied_consumptions,
            dependent_source_facts,
        } = match projected {
            Ok(Ok(completed)) => completed,
            Ok(Err(denial)) => {
                self.graph.release_query_snapshot(&snapshot);
                return Err(denial);
            }
            Err(payload) => {
                self.graph.release_query_snapshot(&snapshot);
                resume_unwind(payload)
            }
        };
        assert!(
            applied_consumptions.is_empty(),
            "committed projections have no candidate inputs"
        );
        Ok(WorthQueryCompletedInvariantProjection {
            output,
            snapshot: WorthQueryApplicationInvariantProjectionSnapshot {
                graph: self.graph.clone(),
                value_bindings: self.value_bindings.clone(),
                layout: Arc::clone(&self.layout),
                basis: Some(basis),
                snapshot: Some(snapshot),
                runtime_authority: self.runtime_authority,
                binding_identity: self.binding_identity.clone(),
                authority_identity: self.authority_identity,
                realized_scope,
                consumed_outputs,
                dependent_source_facts: Some(dependent_source_facts),
                _schema: PhantomData,
            },
            work,
        })
    }
}

impl<Schema, Output> WorthQueryCompletedInvariantProjection<Schema, Output> {
    pub const fn output(&self) -> &Output {
        &self.output
    }

    pub const fn work(&self) -> WorthQueryInvariantProjectionWork {
        self.work
    }

    pub fn into_parts(
        self,
    ) -> (
        Output,
        WorthQueryApplicationInvariantProjectionSnapshot<Schema>,
        WorthQueryInvariantProjectionWork,
    ) {
        (self.output, self.snapshot, self.work)
    }
}

pub mod record_reads;

mod court_doors_for_test;
