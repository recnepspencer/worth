use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

use super::WorthQueryObservedSourceFootprint;
use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::vocabulary::primary_graph::application_query_basis_registry::WorthQueryApplicationBasisSelectionIdentity;

mod checkpoint_commitment;
#[path = "source_identity/tests/epoch_construction.rs"]
mod epoch_construction;
pub(crate) mod identity_kind;
mod interning;
pub(crate) mod material_storage;
pub(in crate::domain_computation::primary_graph::application_query) use material_storage::admit_footprint_comparison;
pub(crate) mod membership;
mod ordering;
mod runtime_commitment;
use crate::domain_computation::primary_graph::vocabulary::application_query::source_identity::WorthQueryObservedSourceCoordinate;
use crate::domain_computation::primary_graph::vocabulary::application_query::source_identity::WorthQueryObservedSourceOccurrence;
pub(in crate::domain_computation::primary_graph) use membership::WorthQueryObservedSourceSelection;
use runtime_commitment::stable_runtime_identity;
use {
    crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryCheckpointSourceIdentity,
    crate::domain_computation::primary_graph::vocabulary::application_query::identity_kind::WorthQueryRuntimeSourceIdentity,
};

/// Runtime-owned exact meaning retained by observed sources and output epochs.
pub struct WorthQueryObservedSourceMeaning {
    coordinate: WorthQueryObservedSourceCoordinate,
    footprint: WorthQueryObservedSourceFootprint,
    identity: [u8; 32],
    runtime_idempotency_identity: [u8; 32],
    checkpoint_identity: [u8; 32],
    registry: Weak<Mutex<WorthQueryObservedSourceMeaningRegistryState>>,
    _material_allocation:
        Option<Arc<crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation>>,
}

impl WorthQueryObservedSourceMeaning {
    pub fn footprint(&self) -> &WorthQueryObservedSourceFootprint {
        &self.footprint
    }

    pub(super) const fn identity(&self) -> &[u8; 32] {
        &self.identity
    }

    pub(super) const fn checkpoint_identity(&self) -> WorthQueryCheckpointSourceIdentity {
        WorthQueryCheckpointSourceIdentity::new(self.checkpoint_identity)
    }

    pub(super) const fn runtime_idempotency_identity(&self) -> WorthQueryRuntimeSourceIdentity {
        WorthQueryRuntimeSourceIdentity::new(self.runtime_idempotency_identity)
    }
}

impl std::fmt::Debug for WorthQueryObservedSourceMeaning {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryObservedSourceMeaning")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

impl Drop for WorthQueryObservedSourceMeaning {
    fn drop(&mut self) {
        let Some(registry) = self.registry.upgrade() else {
            return;
        };
        let mut state = registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remove_coordinate = state
            .meanings
            .get_mut(&self.coordinate)
            .is_some_and(|meanings| {
                meanings
                    .entries
                    .retain(|meaning| meaning.strong_count() > 0);
                meanings.entries.is_empty()
            });
        let retired_coordinate = remove_coordinate.then(|| state.meanings.remove(&self.coordinate));
        let released = state.take_empty_material_storage();
        drop(state);
        drop(released);
        drop(retired_coordinate);
    }
}

pub struct WorthQueryObservedSourceMeaningRegistryState {
    next_ordinal: u64,
    pub meanings:
        BTreeMap<WorthQueryObservedSourceCoordinate, material_storage::CoordinateMeanings>,
}

pub struct WorthQueryObservedSourceMeaningRegistry {
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    state: Arc<Mutex<WorthQueryObservedSourceMeaningRegistryState>>,
}

impl WorthQueryObservedSourceMeaningRegistry {
    pub fn new(runtime_authority: WorthQueryRuntimeAuthorityIdentity) -> Self {
        Self {
            runtime_authority,
            state: Arc::new(Mutex::new(WorthQueryObservedSourceMeaningRegistryState {
                next_ordinal: 1,
                meanings: BTreeMap::new(),
            })),
        }
    }
}

fn source_identity(
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    ordinal: u64,
) -> [u8; 32] {
    let mut identity = [0; 32];
    identity[..8].copy_from_slice(b"WQSRCE01");
    identity[8..16].copy_from_slice(&runtime_authority.as_u64().to_be_bytes());
    identity[16..24].copy_from_slice(&ordinal.to_be_bytes());
    identity
}

/// Complete runtime coordinate of one observed source.
#[derive(Clone, Debug)]
pub struct WorthQueryObservedSourceEpoch {
    query: [u8; 32],
    parameters: [u8; 32],
    root: worth_relational::facade::identity::EntityId,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    observation_generation: u64,
    meaning: Arc<WorthQueryObservedSourceMeaning>,
}

impl WorthQueryObservedSourceEpoch {
    pub(in crate::domain_computation::primary_graph) fn from_observation(
        query: &[u8; 32],
        parameters: &[u8; 32],
        root: worth_relational::facade::identity::EntityId,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
        meaning: Arc<WorthQueryObservedSourceMeaning>,
    ) -> Option<Self> {
        let WorthQueryApplicationBasisSelectionIdentity::Product(product) = selection else {
            return None;
        };
        if meaning.coordinate.query != *query
            || meaning.coordinate.parameters != *parameters
            || meaning.coordinate.root != root
            || meaning.coordinate.occurrence
                != WorthQueryObservedSourceOccurrence::Product(product.lifecycle_incarnation())
        {
            return None;
        }
        Some(Self {
            query: *query,
            parameters: *parameters,
            root,
            occurrence: product.lifecycle_incarnation(),
            observation_generation: product.reference_generation().get(),
            meaning,
        })
    }

    pub fn occurrence_coordinate(&self) -> WorthQueryObservedSourceCoordinate {
        self.meaning.coordinate.clone()
    }

    pub fn same_occurrence(&self, other: &Self) -> bool {
        self.query == other.query
            && self.parameters == other.parameters
            && self.root == other.root
            && self.occurrence == other.occurrence
    }

    pub fn same_semantic_source(&self, other: &Self) -> bool {
        self.same_occurrence(other) && self.meaning.identity() == other.meaning.identity()
    }

    pub(in crate::domain_computation::primary_graph) fn replacement_order(
        &self,
        other: &Self,
    ) -> Option<std::cmp::Ordering> {
        self.same_occurrence(other).then(|| {
            if self.same_semantic_source(other) {
                std::cmp::Ordering::Equal
            } else {
                self.observation_generation
                    .cmp(&other.observation_generation)
            }
        })
    }

    pub(in crate::domain_computation::primary_graph) fn root_entity_for_test(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.root
    }

    pub(in crate::domain_computation::primary_graph) const fn observation_generation(&self) -> u64 {
        self.observation_generation
    }

    pub fn checkpoint_identity(&self) -> WorthQueryCheckpointSourceIdentity {
        self.meaning.checkpoint_identity()
    }

    pub fn runtime_idempotency_identity(&self) -> [u8; 32] {
        self.meaning.runtime_idempotency_identity().bytes()
    }

    pub(in crate::domain_computation::primary_graph) const fn checkpoint_occurrence(
        &self,
    ) -> worth_runtime_world::facade::ProductBranchIncarnation {
        self.occurrence
    }
}

mod court_doors_for_test;
