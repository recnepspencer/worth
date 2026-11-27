//! Crate-boundary ports for invariant projection readers, invariant relations,
//! managed cleanup receipts, and operation body decisions. Never exported on
//! `worth-query-execution::facade` or Host.

use std::cell::Cell;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts;
use worth_relational::facade::identity::{EntityId, RelationId};

use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch;
use crate::domain_computation::authorization::{
    WorthQueryOperationAdmissionIdentity, WorthQueryOperationScopeBinding,
};
use crate::domain_computation::primary_graph::application_attempt::compare_and_commit::cleanup_receipt::{
    RetainedOriginalCleanupCommit, WorthQueryApplicationManagedCleanupReceipt,
    WorthQueryCleanupDispatchPosture,
};
use crate::domain_computation::primary_graph::handler::DecisionContextUse;
use crate::domain_computation::primary_graph::invariant_projection::operation_reader::DecisionReads;
use crate::domain_computation::primary_graph::operation_body::{
    emission::BodyExternalEffectSelection, reference_input::PreparedBodyReferences,
    reuse::CreationDisposition, value::PreparedBodyValues, PreparedBodyChildren,
    PreparedBodyInputs, PreparedBodyPathLinks, PreparedBodyPathWrites, PreparedBodyRetirements,
    PreparedBodySelections, PreparedBodySource, WorthQueryOperationBodyDecision,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationInvariantProjectionReader,
    WorthQueryApplicationOperationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantMutationTarget, WorthQueryInvariantRelation,
};
use crate::ports::SourceQueryReaderPort;

pub trait InvariantProjectionReaderInternalPort<'runtime, Schema> {
    fn native(
        &self,
    ) -> &crate::domain_computation::primary_graph::invariant_projection::read_basis::NativeProjectionRead<'runtime>;
    fn snapshot(&self) -> &'runtime worth_relational::facade::snapshots::SnapshotHandle;
    fn execution(&self) -> &'runtime worth_execution::ExecutionRequest<'runtime, 'runtime>;
    fn retention_control(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<
        'runtime,
        'runtime,
    >;
    fn work_budget(
        &self,
    ) -> &crate::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget;
    fn work_budget_mut(
        &mut self,
    ) -> &mut crate::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget;
    fn work(&self) -> crate::domain_computation::primary_graph::WorthQueryInvariantProjectionWork;
    fn set_work(
        &mut self,
        work: crate::domain_computation::primary_graph::WorthQueryInvariantProjectionWork,
    );
    fn dependent_source_facts_mut(
        &mut self,
    ) -> &mut crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::AuthoringSourceFacts;
    fn selected_product_occurrence(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchIncarnation>;
    fn selected_product_generation(&self) -> Option<u64>;
    fn selected_source_partition_identity(&self) -> Option<[u8; 32]>;
    fn invalidation_owner(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::SourceInvalidationOwner>;
    fn output_lineage(
        &self,
    ) -> &crate::domain_computation::primary_graph::output_lineage::SharedOutputLineage;
    fn native_read(
        &mut self,
    ) -> crate::domain_computation::primary_graph::invariant_projection::native_read::NativeRead<
        '_,
        'runtime,
    >;
}

impl<'runtime, Schema> InvariantProjectionReaderInternalPort<'runtime, Schema>
    for WorthQueryApplicationInvariantProjectionReader<'runtime, Schema>
{
    fn native(
        &self,
    ) -> &crate::domain_computation::primary_graph::invariant_projection::read_basis::NativeProjectionRead<'runtime>
{
        &self.native
    }

    fn snapshot(&self) -> &'runtime worth_relational::facade::snapshots::SnapshotHandle {
        self.snapshot
    }

    fn execution(&self) -> &'runtime worth_execution::ExecutionRequest<'runtime, 'runtime> {
        self.execution
    }

    fn retention_control(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl<
        'runtime,
        'runtime,
    >{
        self.retention_control
    }

    fn work_budget(
        &self,
    ) -> &crate::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget
{
        &self.work_budget
    }

    fn work_budget_mut(
        &mut self,
    ) -> &mut crate::domain_computation::primary_graph::invariant_projection::work::WorthQueryInvariantProjectionWorkBudget
{
        &mut self.work_budget
    }

    fn work(&self) -> crate::domain_computation::primary_graph::WorthQueryInvariantProjectionWork {
        self.work
    }

    fn set_work(
        &mut self,
        work: crate::domain_computation::primary_graph::WorthQueryInvariantProjectionWork,
    ) {
        self.work = work;
    }

    fn dependent_source_facts_mut(
        &mut self,
    ) -> &mut crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::AuthoringSourceFacts
{
        &mut self.dependent_source_facts
    }

    fn selected_product_occurrence(
        &self,
    ) -> Option<worth_runtime_world::facade::ProductBranchIncarnation> {
        self.selected_product_occurrence
    }

    fn selected_product_generation(&self) -> Option<u64> {
        self.selected_product_generation
    }

    fn selected_source_partition_identity(&self) -> Option<[u8; 32]> {
        self.selected_source_partition_identity
    }

    fn invalidation_owner(
        &self,
    ) -> &Arc<crate::domain_computation::primary_graph::SourceInvalidationOwner> {
        &self.invalidation_owner
    }

    fn output_lineage(
        &self,
    ) -> &crate::domain_computation::primary_graph::output_lineage::SharedOutputLineage {
        &self.output_lineage
    }

    fn native_read(
        &mut self,
    ) -> crate::domain_computation::primary_graph::invariant_projection::native_read::NativeRead<
        '_,
        'runtime,
    > {
        Self::native_read(self)
    }
}

pub trait OperationInvariantProjectionReaderInternalPort<'reader, 'runtime, Schema, Operation> {
    fn reader(&self) -> &WorthQueryApplicationInvariantProjectionReader<'runtime, Schema>;
    fn reader_mut(
        &mut self,
    ) -> &mut WorthQueryApplicationInvariantProjectionReader<'runtime, Schema>;
    fn decision_facts(&self) -> &DecisionReads;
    fn decision_facts_mut(&mut self) -> &mut DecisionReads;
    fn source_query(&self) -> Option<&'reader dyn SourceQueryReaderPort>;
    fn source_context_use(&self) -> Option<&'reader Cell<DecisionContextUse>>;
    fn admission_identity(&self) -> Option<WorthQueryOperationAdmissionIdentity>;
    fn operation_scope(&self) -> Option<&WorthQueryOperationScopeBinding>;
    fn admitted_contracts(&self) -> Option<&Arc<WorthQueryCompiledApplicationOperationContracts>>;
    fn selected_product(&self) -> Option<&'reader WorthQueryProductBranchLease>;
    fn admitted_principal(&self) -> Option<EntityId>;
}

impl<'reader, 'runtime, Schema, Operation>
    OperationInvariantProjectionReaderInternalPort<'reader, 'runtime, Schema, Operation>
    for WorthQueryApplicationOperationInvariantProjectionReader<
        'reader,
        'runtime,
        Schema,
        Operation,
    >
{
    fn reader(&self) -> &WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
        self.reader
    }

    fn reader_mut(
        &mut self,
    ) -> &mut WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
        self.reader
    }

    fn decision_facts(&self) -> &DecisionReads {
        self.decision_facts
    }

    fn decision_facts_mut(&mut self) -> &mut DecisionReads {
        self.decision_facts
    }

    fn source_query(&self) -> Option<&'reader dyn SourceQueryReaderPort> {
        self.source_query
    }

    fn source_context_use(&self) -> Option<&'reader Cell<DecisionContextUse>> {
        self.source_context_use
    }

    fn admission_identity(&self) -> Option<WorthQueryOperationAdmissionIdentity> {
        self.admission_identity
    }

    fn operation_scope(&self) -> Option<&WorthQueryOperationScopeBinding> {
        self.operation_scope.as_ref()
    }

    fn admitted_contracts(&self) -> Option<&Arc<WorthQueryCompiledApplicationOperationContracts>> {
        self.admitted_contracts.as_ref()
    }

    fn selected_product(&self) -> Option<&'reader WorthQueryProductBranchLease> {
        self.selected_product
    }

    fn admitted_principal(&self) -> Option<EntityId> {
        self.admitted_principal
    }
}

pub trait InvariantRelationInternalPort<Schema, Relation, From, To>: Sized {
    fn from_parts(
        relation_id: RelationId,
        from: WorthQueryInvariantEntityIdentity<Schema, From>,
        to: WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Self;

    fn into_parts(
        self,
    ) -> (
        RelationId,
        WorthQueryInvariantEntityIdentity<Schema, From>,
        WorthQueryInvariantEntityIdentity<Schema, To>,
    );
}

impl<Schema, Relation, From, To> InvariantRelationInternalPort<Schema, Relation, From, To>
    for WorthQueryInvariantRelation<Schema, Relation, From, To>
{
    fn from_parts(
        relation_id: RelationId,
        from: WorthQueryInvariantEntityIdentity<Schema, From>,
        to: WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Self {
        Self {
            relation_id,
            from,
            to,
            _relation: PhantomData,
        }
    }

    fn into_parts(
        self,
    ) -> (
        RelationId,
        WorthQueryInvariantEntityIdentity<Schema, From>,
        WorthQueryInvariantEntityIdentity<Schema, To>,
    ) {
        (self.relation_id, self.from, self.to)
    }
}

pub trait ManagedCleanupReceiptInternalPort {
    fn original(&self) -> Option<&RetainedOriginalCleanupCommit>;
    fn posture(&self) -> WorthQueryCleanupDispatchPosture;
    fn set_posture(&mut self, posture: WorthQueryCleanupDispatchPosture);
    fn dispatch(&self) -> Option<&WorthQueryExternalEffectDispatch>;
    fn set_dispatch(&mut self, dispatch: Option<WorthQueryExternalEffectDispatch>);
}

impl ManagedCleanupReceiptInternalPort for WorthQueryApplicationManagedCleanupReceipt {
    fn original(&self) -> Option<&RetainedOriginalCleanupCommit> {
        self.original.as_ref()
    }

    fn posture(&self) -> WorthQueryCleanupDispatchPosture {
        self.posture
    }

    fn set_posture(&mut self, posture: WorthQueryCleanupDispatchPosture) {
        self.posture = posture;
    }

    fn dispatch(&self) -> Option<&WorthQueryExternalEffectDispatch> {
        self.dispatch.as_ref()
    }

    fn set_dispatch(&mut self, dispatch: Option<WorthQueryExternalEffectDispatch>) {
        self.dispatch = dispatch;
    }
}

pub struct OperationBodyDecisionParts<Schema, Scope> {
    pub target: WorthQueryInvariantMutationTarget<Schema, Scope>,
    pub inputs: Option<PreparedBodyInputs>,
    pub sources: Vec<Option<PreparedBodySource>>,
    pub selections: PreparedBodySelections<Schema>,
    pub emission: Option<BodyExternalEffectSelection>,
    pub values: Option<PreparedBodyValues>,
    pub retirements: PreparedBodyRetirements<Schema>,
    pub path_writes: PreparedBodyPathWrites<Schema>,
    pub path_links: PreparedBodyPathLinks<Schema>,
    pub principal_link: Option<WorthQueryInvariantMutationTarget<Schema, ()>>,
    pub references: PreparedBodyReferences<Schema>,
    pub creation_disposition: CreationDisposition,
    pub children: Option<PreparedBodyChildren<Schema>>,
    /// The advancement's trusted admission instant, when a creation stamps it.
    pub admitted_at: Option<worth_foundational::facade::CanonicalTimestamp>,
}

pub trait OperationBodyDecisionInternalPort<Schema, Scope>: Sized {
    fn from_parts(parts: OperationBodyDecisionParts<Schema, Scope>) -> Self;
    fn into_parts(self) -> OperationBodyDecisionParts<Schema, Scope>;
}

impl<Schema, Scope> OperationBodyDecisionInternalPort<Schema, Scope>
    for WorthQueryOperationBodyDecision<Schema, Scope>
{
    fn from_parts(parts: OperationBodyDecisionParts<Schema, Scope>) -> Self {
        Self {
            target: parts.target,
            inputs: parts.inputs,
            sources: parts.sources,
            selections: parts.selections,
            emission: parts.emission,
            values: parts.values,
            retirements: parts.retirements,
            path_writes: parts.path_writes,
            path_links: parts.path_links,
            principal_link: parts.principal_link,
            references: parts.references,
            creation_disposition: parts.creation_disposition,
            children: parts.children,
            admitted_at: parts.admitted_at,
        }
    }

    fn into_parts(self) -> OperationBodyDecisionParts<Schema, Scope> {
        OperationBodyDecisionParts {
            target: self.target,
            inputs: self.inputs,
            sources: self.sources,
            selections: self.selections,
            emission: self.emission,
            values: self.values,
            retirements: self.retirements,
            path_writes: self.path_writes,
            path_links: self.path_links,
            principal_link: self.principal_link,
            references: self.references,
            creation_disposition: self.creation_disposition,
            children: self.children,
            admitted_at: self.admitted_at,
        }
    }
}
