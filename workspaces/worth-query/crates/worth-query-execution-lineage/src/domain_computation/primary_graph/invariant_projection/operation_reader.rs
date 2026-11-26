use std::marker::PhantomData;
use std::sync::Arc;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

use worth_query_declaration::facade::application_schema::ApplicationEntityMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationReadableScalarValueBinding, ApplicationRelationRef, ApplicationSchema,
    ApplicationSignedAggregateValueBinding, OperationReads,
    WorthQueryCompiledApplicationOperationContracts,
};

pub mod admitted;
pub mod admitted_projection;
pub mod applied;
pub mod inspection_reader;
pub use applied::AppliedOperationProjection;
pub mod borrowed;
pub mod content;
pub mod fragment;
pub mod fragment_native;
pub mod mutation_target;
pub mod prepared_inputs;
pub use borrowed::BorrowedOperationProjection;
pub use prepared_inputs::PreparedCurrentInputs;
pub mod component_reads;
pub mod computation_calls;
pub mod current_output;
pub use current_output::CertifiedOutputCorrespondence;
pub mod decision_plan;
pub mod decision_reads;
pub mod derived_relation;
pub use derived_relation::{
    WorthQueryInvariantDerivedMemberRelation, WorthQueryInvariantDerivedRelation,
    WorthQueryInvariantDerivedWalk, WorthQueryInvariantDerivedWalkStep,
    WorthQueryInvariantGraphComponent, WorthQueryInvariantGraphMember,
};
pub mod birth_naming;
mod candidate_naming;
pub mod group_naming;
pub mod managed_index;
pub mod name_owner;
pub mod naming;
mod pending_naming;
pub use pending_naming::PendingNameRead;
pub mod relation_naming;
pub use managed_index::{
    native_index_decision_cause, WorthQueryComputationManagedIndexDenial,
    WorthQueryComputationManagedIndexDenialKind,
};
pub mod output_availability;
pub mod shared_owner_scope;
pub use output_availability::{
    WorthQueryObservedOutputAvailability, WorthQueryOutputAvailabilityDenial,
    WorthQueryOutputAvailabilityState,
};
pub use shared_owner_scope::SharedOwnerRelation;
pub mod prior_output;

pub use computation_calls::{
    CallSequenceDenial, CallShape, CallShapeTransition, CallVisit, ComputationCallCharge,
    ComputationCallSequence,
};
pub use current_output::{
    WorthQueryCurrentOutputDenial, WorthQueryCurrentOutputDenialKind,
    WorthQueryCurrentOutputSelection,
};
pub use decision_plan::{
    WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantDecisionPlanDenialKind,
    WorthQueryPreparedEntitySelection,
};
pub use decision_reads::{DecisionReads, ExpectedFactKeys, ObservedComputationInputs};
pub use prior_output::WorthQueryPriorOutputFamilyMember;
pub use prior_output::WorthQueryPriorOutputRead;

use super::super::request_local::{assert_request_local, RequestLocal};
use super::{
    WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryApplicationInvariantProjectionReader,
    WorthQueryApplicationInvariantProjectionSnapshot, WorthQueryCompletedInvariantProjection,
    WorthQueryInvariantAggregateDenial, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantMutationTarget, WorthQueryInvariantProjectionDenial,
    WorthQueryInvariantProjectionTraversalDenial, WorthQueryInvariantProjectionWork,
    WorthQueryInvariantRelation, WorthQueryOperationProjectionDenial,
};
use crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity;
use crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial;
use crate::ports::{AdmittedOperationPort, SourceQueryReaderPort};

/// An invariant projection reader restricted to one operation's declared reads.
///
/// Decision reads record the facts the operation's outcome depends on. Only a
/// reader opened for an admitted operation can mint mutation targets.
pub struct WorthQueryApplicationOperationInvariantProjectionReader<
    'reader,
    'runtime,
    Schema,
    Operation,
> {
    pub(crate) reader:
        &'reader mut WorthQueryApplicationInvariantProjectionReader<'runtime, Schema>,
    pub(crate) source_query: Option<&'reader dyn SourceQueryReaderPort>,
    pub(crate) source_context_use: Option<
        &'reader std::cell::Cell<
            crate::domain_computation::primary_graph::handler::DecisionContextUse,
        >,
    >,
    pub(crate) admitted_contracts: Option<Arc<WorthQueryCompiledApplicationOperationContracts>>,
    pub(crate) runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    pub(crate) binding_identity:
        &'reader worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    pub(crate) admission_identity: Option<WorthQueryOperationAdmissionIdentity>,
    pub(crate) selected_product: Option<&'reader crate::basis::WorthQueryProductBranchLease>,
    pub(crate) admitted_root: Option<admitted::AdmittedReadRoot>,
    pub(crate) admitted_principal: Option<worth_relational::facade::identity::EntityId>,
    pub(crate) query_context:
        Option<&'reader super::super::application_query::ObservedQueryExpressionContext>,
    pub(crate) operation_scope:
        Option<crate::domain_computation::authorization::WorthQueryOperationScopeBinding>,
    pub(crate) decision_facts: &'reader mut DecisionReads,
    _operation: PhantomData<fn() -> Operation>,
    request_local: RequestLocal,
}

assert_request_local!(
    WorthQueryApplicationOperationInvariantProjectionReader<'static, 'static, (), ()>,
);

/// The result of projecting for an admitted operation: the output, the work
/// spent, and a snapshot typed for that operation.
///
/// Pass the snapshot from `into_parts` to
/// `begin_projected_application_read_attempt`.
pub struct WorthQueryCompletedOperationInvariantProjection<Schema, Operation, Output> {
    completed: WorthQueryCompletedInvariantProjection<Schema, (Output, DecisionReads)>,
    admission_identity: WorthQueryOperationAdmissionIdentity,
    product: crate::basis::WorthQueryProductBranchLease,
    _operation: PhantomData<fn() -> Operation>,
}

/// The result of an operation-typed inspection: the output and the work spent.
///
/// It keeps no snapshot and grants no mutation authority.
pub struct WorthQueryInspectedOperationInvariantProjection<Operation, Output> {
    output: Output,
    work: WorthQueryInvariantProjectionWork,
    _operation: PhantomData<fn() -> Operation>,
}

/// A projection snapshot bound to one admitted operation, carrying the decision
/// facts it read.
///
/// Only a read attempt for that same operation accepts it.
pub struct WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation> {
    snapshot: WorthQueryApplicationInvariantProjectionSnapshot<Schema>,
    admission_identity: WorthQueryOperationAdmissionIdentity,
    product: crate::basis::WorthQueryProductBranchLease,
    decision_facts: DecisionReads,
    _operation: PhantomData<fn() -> Operation>,
}

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>
where
    Schema: ApplicationSchema,
{
    /// Runs an operation-typed inspection without retaining mutation authority.
    ///
    /// The inspection result has no snapshot extraction transition:
    pub fn project_operation<Operation, Output>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryInspectedOperationInvariantProjection<Operation, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        let completed = self.project(
            phase,
            |reader| self.inspect_reader(reader, None, None, projection),
            allocation_policy,
        )?;
        let ((output, _), snapshot, work) = completed.into_parts();
        drop(snapshot);
        Ok(WorthQueryInspectedOperationInvariantProjection {
            output,
            work,
            _operation: PhantomData,
        })
    }

    pub(crate) fn project_operation_on_product<Operation, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &crate::basis::WorthQueryProductBranchLease,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryInspectedOperationInvariantProjection<Operation, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        let completed = self.project_with_work_limit(
            execution,
            None,
            product.relational_basis().clone(),
            |reader| self.inspect_reader(reader, None, Some(product), projection),
            None,
            allocation_policy,
        )?;
        let ((output, _), snapshot, work) = completed.into_parts();
        drop(snapshot);
        Ok(WorthQueryInspectedOperationInvariantProjection {
            output,
            work,
            _operation: PhantomData,
        })
    }

    pub(crate) fn project_admitted_operation_port<Operation, Scope, Output>(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        admission: &dyn AdmittedOperationPort,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
            &WorthQueryInvariantEntityIdentity<Schema, Scope>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompletedOperationInvariantProjection<Schema, Operation, Output>,
        WorthQueryOperationProjectionDenial,
    > {
        if let Some(owner) = phase.world_owner() {
            admission.refuse_stale_home_authority(owner)?;
        }
        admission.validate_projection_authority(
            self.runtime_authority,
            &self.binding_identity,
            &self.installed_operations,
        )?;
        let owner = admission
            .mutation_product()
            .expect("admitted operation retains its selected mutation lease")
            .observation()
            .owner_identity();
        let phase_request = phase.request_for_owner(owner).map_err(|_| {
            WorthQueryOperationProjectionDenial::from_invariant(
                super::WorthQueryInvariantProjectionDenial::from_kind(
                    super::WorthQueryInvariantProjectionDenialKind::ForeignPhase,
                ),
                admission.operation(),
            )
        })?;
        self.project_admitted_operation_with_request_port(
            &phase_request,
            admission,
            projection,
            allocation_policy,
        )
    }

    pub(crate) fn project_admitted_operation_with_request_port<Operation, Scope, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &dyn AdmittedOperationPort,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
            &WorthQueryInvariantEntityIdentity<Schema, Scope>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryCompletedOperationInvariantProjection<Schema, Operation, Output>,
        WorthQueryOperationProjectionDenial,
    > {
        self.project_admitted_operation_with_source_access_port(
            execution,
            admission,
            projection,
            allocation_policy,
            None,
            None,
        )
    }
}

impl<Operation, Output> WorthQueryInspectedOperationInvariantProjection<Operation, Output> {
    pub const fn output(&self) -> &Output {
        &self.output
    }

    pub const fn work(&self) -> WorthQueryInvariantProjectionWork {
        self.work
    }

    pub fn into_output(self) -> Output {
        self.output
    }
}

impl<Schema, Operation, Output>
    WorthQueryCompletedOperationInvariantProjection<Schema, Operation, Output>
{
    pub const fn output(&self) -> &Output {
        &self.completed.output().0
    }

    pub const fn work(&self) -> WorthQueryInvariantProjectionWork {
        self.completed.work()
    }

    pub fn into_parts(
        self,
    ) -> (
        Output,
        WorthQueryApplicationOperationInvariantProjectionSnapshot<Schema, Operation>,
        WorthQueryInvariantProjectionWork,
    ) {
        let ((output, decision_facts), snapshot, work) = self.completed.into_parts();
        (
            output,
            WorthQueryApplicationOperationInvariantProjectionSnapshot {
                snapshot,
                admission_identity: self.admission_identity,
                product: self.product,
                decision_facts,
                _operation: PhantomData,
            },
            work,
        )
    }
}

#[path = "operation_reader/read_access.rs"]
pub mod read_access;

pub mod snapshot_scope;

mod court_doors_for_test;
