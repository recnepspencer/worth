//! Crate-boundary internal ports for document history node identity and preview
//! values consumed by `worth-query-execution`. Never exported on
//! `worth-query-execution::facade` or Host.

use std::sync::Arc;

use super::discovery_recipes::HistoryDiscoveryRecipes;
use super::step::DocumentIncarnation;
use super::{
    HistoryModelIdentity, WorthQueryHistoryNodeId, WorthQueryHistoryPreview, WorthQueryHistoryStep,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationReadObservation, WorthQueryOutputSettlementPosture,
};

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
