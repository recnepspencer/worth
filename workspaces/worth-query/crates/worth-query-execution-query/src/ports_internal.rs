//! Crate-boundary internal ports for `worth-query-execution-query` types
//! consumed strictly by `worth-query-execution`. Never exported on
//! `worth-query-execution::facade` or Host.

use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_execution_runtime::facade::execution_runtime as rt;
use worth_query_installation::facade::WorthQueryCanonicalWorkEvidence;

use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntime;
use crate::domain_computation::primary_graph::application_query::{
    WorthQueryApplicationBasisIdentity, WorthQueryObservedSource,
};
use crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationQueryObservation;
use crate::domain_computation::primary_graph::conditional_operation::installation::clock_handle::{
    ConditionalClockLease, WorthQueryConditionalClockHandle,
};
use crate::domain_computation::primary_graph::product_operation::document_history::discovery_recipes::HistoryDiscoveryRecipes;
use crate::domain_computation::primary_graph::product_operation::document_history::step::DocumentIncarnation;
use crate::domain_computation::primary_graph::product_operation::document_history::{
    HistoryModelIdentity, WorthQueryHistoryNodeId, WorthQueryHistoryPreview, WorthQueryHistoryStep,
};
use crate::domain_computation::primary_graph::product_operation::WorthQueryProductQueryControls;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationReadObservation, WorthQueryOutputSettlementPosture, WorthQueryPrimaryGraph,
};

pub trait QueryExecutionRuntimeInternalPort: Sized {
    fn from_inner(inner: rt::WorthQueryExecutionRuntime) -> Self;
    fn primary_graph_mut(&mut self) -> Option<&mut WorthQueryPrimaryGraph>;
    fn install_primary_graph(&mut self, graph: WorthQueryPrimaryGraph);
}

impl QueryExecutionRuntimeInternalPort for WorthQueryExecutionRuntime {
    fn from_inner(inner: rt::WorthQueryExecutionRuntime) -> Self {
        Self::from_inner(inner)
    }

    fn primary_graph_mut(&mut self) -> Option<&mut WorthQueryPrimaryGraph> {
        Self::primary_graph_mut(self)
    }

    fn install_primary_graph(&mut self, graph: WorthQueryPrimaryGraph) {
        Self::install_primary_graph(self, graph)
    }
}

pub trait ConditionalClockHandleInternalPort<Schema, Node, Clock>: Sized {
    fn from_parts(
        binding_identity: Arc<str>,
        binding_identity_digest: [u8; 32],
        node_authority: Arc<str>,
        binding_canonical_work: WorthQueryCanonicalWorkEvidence,
        lease: Arc<ConditionalClockLease>,
    ) -> Self;

    fn new_lease() -> Arc<ConditionalClockLease>;
}

impl<Schema, Node, Clock> ConditionalClockHandleInternalPort<Schema, Node, Clock>
    for WorthQueryConditionalClockHandle<Schema, Node, Clock>
{
    fn from_parts(
        binding_identity: Arc<str>,
        binding_identity_digest: [u8; 32],
        node_authority: Arc<str>,
        binding_canonical_work: WorthQueryCanonicalWorkEvidence,
        lease: Arc<ConditionalClockLease>,
    ) -> Self {
        Self {
            binding_identity,
            binding_identity_digest,
            node_authority,
            binding_canonical_work,
            lease,
            marker: PhantomData,
        }
    }

    fn new_lease() -> Arc<ConditionalClockLease> {
        Arc::new(ConditionalClockLease)
    }
}

pub trait ApplicationQueryObservationInternalPort: Sized {
    fn from_parts(
        source: WorthQueryObservedSource<()>,
        basis: WorthQueryApplicationBasisIdentity,
        result_count: usize,
    ) -> Self;
    fn source(&self) -> &WorthQueryObservedSource<()>;
    fn basis(&self) -> &WorthQueryApplicationBasisIdentity;
    fn result_count(&self) -> usize;
}

impl ApplicationQueryObservationInternalPort for WorthQueryApplicationQueryObservation {
    fn from_parts(
        source: WorthQueryObservedSource<()>,
        basis: WorthQueryApplicationBasisIdentity,
        result_count: usize,
    ) -> Self {
        Self {
            source,
            basis,
            result_count,
        }
    }

    fn source(&self) -> &WorthQueryObservedSource<()> {
        &self.source
    }

    fn basis(&self) -> &WorthQueryApplicationBasisIdentity {
        &self.basis
    }

    fn result_count(&self) -> usize {
        self.result_count
    }
}

pub trait HistoryNodeIdInternalPort: Sized {
    fn from_parts(document: DocumentIncarnation, ordinal: u64) -> Self;
    fn document(&self) -> DocumentIncarnation;
    fn ordinal(&self) -> u64;
}

impl HistoryNodeIdInternalPort for WorthQueryHistoryNodeId {
    fn from_parts(document: DocumentIncarnation, ordinal: u64) -> Self {
        Self { document, ordinal }
    }

    fn document(&self) -> DocumentIncarnation {
        self.document
    }

    fn ordinal(&self) -> u64 {
        self.ordinal
    }
}

pub trait HistoryPreviewInternalPort: Sized {
    fn from_parts(
        discovery_recipes: Option<Arc<HistoryDiscoveryRecipes>>,
        step: WorthQueryHistoryStep,
        model: HistoryModelIdentity,
        source: Arc<WorthQueryApplicationReadObservation>,
        root_posture: Option<WorthQueryOutputSettlementPosture>,
    ) -> Self;
    fn discovery_recipes(&self) -> Option<&Arc<HistoryDiscoveryRecipes>>;
    fn step(&self) -> &WorthQueryHistoryStep;
    fn model(&self) -> &HistoryModelIdentity;
    fn source(&self) -> &Arc<WorthQueryApplicationReadObservation>;
    fn root_posture(&self) -> Option<WorthQueryOutputSettlementPosture>;
}

impl HistoryPreviewInternalPort for WorthQueryHistoryPreview {
    fn from_parts(
        discovery_recipes: Option<Arc<HistoryDiscoveryRecipes>>,
        step: WorthQueryHistoryStep,
        model: HistoryModelIdentity,
        source: Arc<WorthQueryApplicationReadObservation>,
        root_posture: Option<WorthQueryOutputSettlementPosture>,
    ) -> Self {
        Self {
            discovery_recipes,
            step,
            model,
            source,
            root_posture,
        }
    }

    fn discovery_recipes(&self) -> Option<&Arc<HistoryDiscoveryRecipes>> {
        self.discovery_recipes.as_ref()
    }

    fn step(&self) -> &WorthQueryHistoryStep {
        &self.step
    }

    fn model(&self) -> &HistoryModelIdentity {
        &self.model
    }

    fn source(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.source
    }

    fn root_posture(&self) -> Option<WorthQueryOutputSettlementPosture> {
        self.root_posture
    }
}

pub trait ProductQueryControlsInternalPort<'request> {
    fn maximum_results(&self) -> NonZeroUsize;
    fn request(&self) -> &'request WorthQueryRequestScope;
}

impl<'request> ProductQueryControlsInternalPort<'request>
    for WorthQueryProductQueryControls<'request>
{
    fn maximum_results(&self) -> NonZeroUsize {
        self.maximum_results
    }

    fn request(&self) -> &'request WorthQueryRequestScope {
        self.request
    }
}

pub trait BatchAdmissionInternalPort {
    fn take_planned_read(
        &self,
        runtime: u64,
        schema: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        query: &worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
        maximum_roots: usize,
    ) -> Result<
        Option<crate::domain_computation::primary_graph::application_query::batch::PlannedBatchItem>,
        crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchResourceDenial,
    >;
}

impl BatchAdmissionInternalPort
    for crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchAdmission
{
    fn take_planned_read(
        &self,
        runtime: u64,
        schema: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        query: &worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity,
        maximum_roots: usize,
    ) -> Result<
        Option<crate::domain_computation::primary_graph::application_query::batch::PlannedBatchItem>,
        crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchResourceDenial,
    > {
        Self::take_planned_read(self, runtime, schema, query, maximum_roots)
    }
}
