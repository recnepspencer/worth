mod output_availability_candidate;
mod retention;
pub(in crate::domain_computation::primary_graph) use output_availability_candidate::ObservedOutputCandidate;
mod admission_denial;
mod aggregate;
mod aggregate_read;
mod component;
mod graph_member;
mod logical_projection;
pub use component::WorthQueryInvariantProjectionComponent;
pub(crate) mod consumed_output;
mod entity_resident;
#[path = "invariant_projection/field_projection.rs"]
mod field_projection;
mod identity_encoding;
pub(crate) mod pending_output_read;
pub(in crate::domain_computation::primary_graph) use pending_output_read::RequestedOutputRead;
mod inventory;
mod lineage;
mod orbit;
pub use orbit::WorthQueryInvariantProjectionOrbit;
mod locked_reader;
mod operation_projection_denial;
pub(crate) mod operation_reader;
pub(in crate::domain_computation::primary_graph) use operation_reader::native_index_decision_cause;
pub(in crate::domain_computation::primary_graph) use operation_reader::BorrowedOperationProjection;
pub use operation_reader::{
    WorthQueryInvariantDerivedMemberRelation, WorthQueryInvariantDerivedRelation,
    WorthQueryInvariantDerivedWalk, WorthQueryInvariantDerivedWalkStep,
    WorthQueryInvariantGraphComponent, WorthQueryInvariantGraphMember,
    WorthQueryObservedOutputAvailability, WorthQueryOutputAvailabilityDenial,
    WorthQueryOutputAvailabilityState,
};
pub(crate) mod realized_scope;
mod relation_field;
mod relation_identity;
pub use relation_identity::WorthQueryInvariantRelation;
mod traversal;
mod walk;
pub(crate) mod work;
pub(in crate::domain_computation::primary_graph) use consumed_output::EvidenceView;
pub use walk::{
    WorthQueryInvariantProjectionWalk, WorthQueryInvariantProjectionWalkIter,
    WorthQueryInvariantProjectionWalkStep,
};

use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationReadableScalarValueBinding,
    ApplicationSchema, ApplicationSchemaBindingIdentity, DeclaredApplicationFieldValue,
    WritePosture,
};
use worth_relational::facade::identity::{EntityId, KindId, VersionId};

use crate::domain_computation::graph_layout::WorthQueryPrimaryGraphLayout;
use super::{
    WorthQueryInstalledEntityResolutionContext, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphIntegrationHandle,
};
use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;

pub use super::read_observation::WorthQueryInvariantProjectionWork;
pub use admission_denial::{
    WorthQueryInvariantProjectionDenial, WorthQueryInvariantProjectionDenialKind,
};
pub use aggregate::{
    WorthQueryInvariantAggregate, WorthQueryInvariantAggregateDenial,
    WorthQueryInvariantAggregateDenialKind,
};

pub use field_projection::WorthQueryRetainedSnapshotRead;
pub use locked_reader::{
    WorthQueryApplicationInvariantProjectionReader, WorthQueryCompletedInvariantProjection,
    WorthQueryInvariantProjectionTraversalDenial, WorthQueryInvariantProjectionTraversalDenialKind,
};
pub use operation_projection_denial::{
    WorthQueryOperationProjectionDenial, WorthQueryOperationProjectionDenialKind,
};
pub(in crate::domain_computation::primary_graph) use operation_reader::WorthQueryPriorOutputRead;
pub(in crate::domain_computation::primary_graph) use operation_reader::{
    CallSequenceDenial, CallShape, CallShapeTransition, CallVisit, ComputationCallCharge,
    ComputationCallSequence, ExpectedFactKeys, ObservedComputationInputs, PreparedCurrentInputs,
};
pub use operation_reader::{
    WorthQueryApplicationOperationInvariantProjectionReader,
    WorthQueryApplicationOperationInvariantProjectionSnapshot,
    WorthQueryCompletedOperationInvariantProjection, WorthQueryCurrentOutputDenial,
    WorthQueryCurrentOutputDenialKind, WorthQueryCurrentOutputSelection,
    WorthQueryInspectedOperationInvariantProjection, WorthQueryInvariantDecisionPlanDenial,
    WorthQueryInvariantDecisionPlanDenialKind, WorthQueryPreparedEntitySelection,
    WorthQueryPriorOutputFamilyMember,
};
pub(in crate::domain_computation::primary_graph) use realized_scope::WorthQueryRealizedProjectionScope;
pub(in crate::domain_computation::primary_graph) use {
    consumed_output::ConsumedOutputEvidence, consumed_output::SelectedPendingConsumedOutput,
};

static NEXT_INVARIANT_PROJECTION_AUTHORITY: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(1);

/// Installation-minted authority for domain invariant projection.
///
/// The application composition root may retain this while it owns the
/// move-only primary-graph bootstrap. Ordinary consumers cannot construct it
/// from runtime identities or application admissions.
pub struct WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    installed_operations: Arc<worth_query_installation::facade::InstalledOperationCatalog>,
    graph: WorthQueryPrimaryGraphIntegrationHandle,
    layout: Arc<WorthQueryPrimaryGraphLayout>,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
    entity_resolution: WorthQueryInstalledEntityResolutionContext,
    authority_identity: u64,
    value_bindings:
        worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    program_activation: Option<super::program_occurrence::WorthQueryProgramActivationCell>,
    _schema: PhantomData<fn() -> Schema>,
}

/// A retained read of the main branch at one version, kept after an invariant
/// projection completes.
///
/// `version` names the version read and `field` reads further values at it.
/// Dropping the snapshot releases it.
pub struct WorthQueryApplicationInvariantProjectionSnapshot<Schema> {
    graph: WorthQueryPrimaryGraphIntegrationHandle,
    value_bindings:
        worth_query_installation::facade::WorthQueryInstalledApplicationValueBindingCatalog,
    layout: Arc<WorthQueryPrimaryGraphLayout>,
    basis: Option<worth_relational::facade::branch::AdmittedRelationalBranchBasis>,
    snapshot: Option<worth_relational::facade::snapshots::SnapshotHandle>,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
    authority_identity: u64,
    realized_scope: WorthQueryRealizedProjectionScope,
    consumed_outputs:
        BTreeMap<Arc<crate::domain_computation::primary_graph::vocabulary::output_lineage::settlement_identity::RecordedSettlementIdentity>, ConsumedOutputEvidence>,
    dependent_source_facts:
        Option<super::application_attempt::retained_decision_facts::RetainedSourceFacts>,
    _schema: PhantomData<fn() -> Schema>,
}

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation::primary_graph) fn belongs_to_installation(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.runtime_authority == runtime_authority && &self.binding_identity == binding_identity
    }

    pub(in crate::domain_computation::primary_graph) fn active_snapshot_count(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> usize {
        self.graph.with_runtime_mut(request_execution, |runtime| {
            runtime.retention().active_snapshot_count()
        })
    }

    pub fn snapshot(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
    ) -> Result<
        WorthQueryApplicationInvariantProjectionSnapshot<Schema>,
        WorthQueryInvariantProjectionDenial,
    > {
        let request = phase
            .request_for_source(self.graph.relational_runtime_instance_id())
            .map_err(|_| {
                WorthQueryInvariantProjectionDenial::from_kind(
                    WorthQueryInvariantProjectionDenialKind::ForeignPhase,
                )
            })?;
        let execution = &request;
        let (basis, snapshot) = self.graph.with_runtime_mut(execution, |runtime| {
            let identity = runtime.main_branch_identity();
            let (_, basis) = runtime
                .observe_branch(&identity)
                .map_err(admission_denial::from_branch_basis_denial)?;
            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .map_err(admission_denial::from_snapshot_admission_denial)?;
            Ok((basis, snapshot))
        })?;
        Ok(WorthQueryApplicationInvariantProjectionSnapshot {
            graph: self.graph.clone(),
            value_bindings: self.value_bindings.clone(),
            layout: Arc::clone(&self.layout),
            basis: Some(basis),
            snapshot: Some(snapshot),
            runtime_authority: self.runtime_authority,
            binding_identity: self.binding_identity.clone(),
            authority_identity: self.authority_identity,
            realized_scope: WorthQueryRealizedProjectionScope::default(),
            consumed_outputs: BTreeMap::new(),
            dependent_source_facts: None,
            _schema: PhantomData,
        })
    }
}

impl<Schema> WorthQueryApplicationInvariantProjectionSnapshot<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn version(&self) -> VersionId {
        self.snapshot().version_id()
    }

    pub(in crate::domain_computation::primary_graph) fn belongs_to(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        self.runtime_authority == runtime_authority && &self.binding_identity == binding_identity
    }

    pub(in crate::domain_computation::primary_graph) fn into_lease(
        mut self,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> super::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease {
        let basis = self
            .basis
            .take()
            .expect("projection snapshot retains its owner-admitted basis");
        let snapshot = self
            .snapshot
            .take()
            .expect("projection snapshot remains live until consumed");
        super::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease::from_existing(
            self.graph.clone(),
            Arc::clone(&self.layout),
            basis,
            snapshot,
            product,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn into_lease_and_realized_scope(
        mut self,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> (
        super::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease,
        WorthQueryRealizedProjectionScope,
        Option<super::application_attempt::retained_decision_facts::RetainedSourceFacts>,
        Vec<ConsumedOutputEvidence>,
    ) {
        let realized_scope = std::mem::take(&mut self.realized_scope);
        let dependent_source_facts = self.dependent_source_facts.take();
        let consumed_outputs = std::mem::take(&mut self.consumed_outputs)
            .into_values()
            .collect();
        (
            self.into_lease(product),
            realized_scope,
            dependent_source_facts,
            consumed_outputs,
        )
    }

    fn snapshot(&self) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.snapshot
            .as_ref()
            .expect("projection snapshot remains live until consumed")
    }
}

impl<Schema> Drop for WorthQueryApplicationInvariantProjectionSnapshot<Schema> {
    fn drop(&mut self) {
        if let Some(snapshot) = self.snapshot.take() {
            self.graph.release_query_snapshot(&snapshot);
        }
    }
}

mod court_doors_for_test;
pub use crate::domain_computation::vocabulary::primary_graph::value_decode::WorthQueryValueDecodeDenial;
