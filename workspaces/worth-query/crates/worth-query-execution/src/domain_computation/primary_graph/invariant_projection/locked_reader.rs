use std::collections::{BTreeMap, HashMap};
use std::marker::PhantomData;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationSchema, DeclaredApplicationFieldValue,
    EqualityPredicate, WritePosture,
};

use super::work::WorthQueryInvariantProjectionWorkBudget;
use super::{
    WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryApplicationInvariantProjectionSnapshot, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionDenial, WorthQueryInvariantProjectionWork,
    WorthQueryRealizedProjectionScope,
};
use crate::domain_computation::primary_graph::{
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind,
    WorthQueryPrincipalResolutionMode,
};

#[path = "locked_reader/predecode_admission.rs"]
mod predecode_admission;
#[path = "locked_reader/traversal_denial.rs"]
mod traversal_denial;
pub use traversal_denial::{
    WorthQueryInvariantProjectionTraversalDenial, WorthQueryInvariantProjectionTraversalDenialKind,
};

/// The reader a projection closure uses to read the main branch at one
/// snapshot.
///
/// It resolves entities by identity field, reads fields, traverses relations,
/// and summarizes aggregates. Every read is charged to the projection's work.
pub struct WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
    pub(super) runtime: &'runtime mut worth_relational::facade::runtime::RelationalRuntime,
    pub(super) layout: &'runtime super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    pub(super) snapshot: &'runtime worth_relational::facade::snapshots::SnapshotHandle,
    entity_resolution: &'runtime super::super::WorthQueryInstalledEntityResolutionContext,
    pub(super) authority_identity: u64,
    pub(super) work: WorthQueryInvariantProjectionWork,
    pub(super) work_budget: WorthQueryInvariantProjectionWorkBudget,
    pub(super) realized_scope: WorthQueryRealizedProjectionScope,
    pub(super) aggregate_projections:
        Arc<std::sync::Mutex<super::super::aggregate_projection::WorthQueryAggregateProjections>>,
    pub(super) output_lineage:
        Arc<std::sync::Mutex<super::super::output_lineage::WorthQueryApplicationOutputLineage>>,
    pub(super) invalidation_owner: Arc<super::super::SourceInvalidationOwner>,
    pub(super) selected_product_occurrence:
        Option<worth_runtime_world::facade::ProductBranchIncarnation>,
    pub(super) selected_product_generation: Option<u64>,
    pub(super) selected_source_partition_identity: Option<[u8; 32]>,
    pub(super) prior_output_bindings:
        HashMap<std::any::TypeId, Arc<super::super::WorthQueryApplicationOutputCorrespondence>>,
    pub(super) current_output_families: HashMap<
        (
            std::any::TypeId,
            worth_relational::facade::identity::EntityId,
        ),
        Vec<(
            Arc<super::super::WorthQueryApplicationOutputCorrespondence>,
            String,
        )>,
    >,
    pub(super) dependent_source_facts:
        super::super::application_attempt::retained_decision_facts::AuthoringSourceFacts,
    pub(super) retention_control:
        super::super::application_attempt::retained_decision_facts::StorageControl<
            'runtime,
            'runtime,
        >,
    pub(super) consumed_outputs: BTreeMap<
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
        projection: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompletedInvariantProjection<Schema, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        let basis = self.graph.with_runtime_mut(|runtime| {
            let identity = runtime.main_branch_identity();
            runtime
                .observe_branch(&identity)
                .map(|(_, basis)| basis)
                .map_err(super::admission_denial::from_branch_basis_denial)
        })?;
        self.project_with_work_budget(
            WorthQueryInvariantProjectionWorkBudget::unbounded(),
            basis,
            projection,
            super::super::application_attempt::retained_decision_facts::StorageControl::new(
                allocation_policy,
                None,
            ),
        )
    }

    pub(super) fn project_bounded<Output>(
        &self,
        maximum_work: usize,
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
            WorthQueryInvariantProjectionWorkBudget::bounded(maximum_work),
            basis,
            projection,
            super::super::application_attempt::retained_decision_facts::StorageControl::new(
                allocation_policy,
                request,
            ),
        )
    }

    fn project_with_work_budget<Output>(
        &self,
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
        let snapshot = self.graph.with_runtime_mut(|runtime| {
            runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .map_err(super::admission_denial::from_snapshot_admission_denial)
        })?;
        let projected = self.graph.with_runtime_mut(|runtime| {
            catch_unwind(AssertUnwindSafe(|| {
                let mut reader = WorthQueryApplicationInvariantProjectionReader {
                    runtime,
                    layout: &self.layout,
                    snapshot: &snapshot,
                    entity_resolution: &self.entity_resolution,
                    authority_identity: self.authority_identity,
                    work: WorthQueryInvariantProjectionWork::default(),
                    work_budget,
                    realized_scope: WorthQueryRealizedProjectionScope::default(),
                    aggregate_projections: Arc::clone(&self.graph.aggregate_projections),
                    output_lineage: Arc::clone(&self.graph.output_lineage),
                    invalidation_owner: Arc::clone(&self.graph.source_owner.invalidation_owner),
                    selected_product_occurrence: None,
                    selected_product_generation: None,
                    selected_source_partition_identity: None,
                    prior_output_bindings: HashMap::new(),
                    current_output_families: HashMap::new(),
                    consumed_outputs: BTreeMap::new(),
                    dependent_source_facts: super::super::application_attempt::retained_decision_facts::AuthoringSourceFacts::vacant(),
                    retention_control,
                    _schema: PhantomData,
                };
                let output = projection(&mut reader);
                (
                    output,
                    reader.work,
                    reader.realized_scope,
                    reader.consumed_outputs,
                    reader.dependent_source_facts.finish(retention_control),
                    reader.work_budget.exceeded(),
                )
            }))
        });
        let (output, work, realized_scope, consumed_outputs, dependent_source_facts, exceeded) =
            match projected {
                Ok(completed) => completed,
                Err(payload) => {
                    self.graph.with_runtime_mut(|runtime| {
                        crate::relational_snapshot_release::release_query_snapshot(
                            runtime, &snapshot,
                        );
                    });
                    resume_unwind(payload)
                }
            };
        if exceeded {
            self.graph.with_runtime_mut(|runtime| {
                crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            });
            let denial = WorthQueryInvariantProjectionDenial::work_budget_exceeded(work);
            return Err(match dependent_source_facts {
                Ok(_) => denial,
                Err(retention) => denial.with_retention_denial(retention),
            });
        }
        let dependent_source_facts = match dependent_source_facts {
            Ok(facts) => facts,
            Err(denial) => {
                self.graph.with_runtime_mut(|runtime| {
                    crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
                });
                return Err(
                    WorthQueryInvariantProjectionDenial::source_retention_denied(denial, work),
                );
            }
        };
        Ok(WorthQueryCompletedInvariantProjection {
            output,
            snapshot: WorthQueryApplicationInvariantProjectionSnapshot {
                graph: self.graph.clone(),
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

mod record_reads;
