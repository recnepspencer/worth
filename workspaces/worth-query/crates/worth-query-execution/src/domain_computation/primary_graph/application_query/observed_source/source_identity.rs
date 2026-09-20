use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, Weak};

use super::super::resource_lifecycle::WorthQueryApplicationBasisSelectionIdentity;
use super::WorthQueryObservedSourceFootprint;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum WorthQueryObservedSourceOccurrence {
    Relational,
    Product(worth_runtime_world::facade::ProductBranchIncarnation),
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct WorthQueryObservedSourceCoordinate {
    query: [u8; 32],
    parameters: [u8; 32],
    occurrence: WorthQueryObservedSourceOccurrence,
    root: worth_relational::facade::identity::EntityId,
}

/// Runtime-owned exact meaning retained by observed sources and output epochs.
pub(in crate::domain_computation::primary_graph) struct WorthQueryObservedSourceMeaning {
    coordinate: WorthQueryObservedSourceCoordinate,
    footprint: WorthQueryObservedSourceFootprint,
    identity: [u8; 32],
    checkpoint_identity: [u8; 32],
    registry: Weak<Mutex<WorthQueryObservedSourceMeaningRegistryState>>,
}

impl WorthQueryObservedSourceMeaning {
    pub(super) fn footprint(&self) -> &WorthQueryObservedSourceFootprint {
        &self.footprint
    }

    pub(super) const fn identity(&self) -> &[u8; 32] {
        &self.identity
    }

    pub(super) const fn checkpoint_identity(&self) -> [u8; 32] {
        self.checkpoint_identity
    }

    pub(super) const fn durable_idempotency_identity(&self) -> [u8; 32] {
        self.checkpoint_identity
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
                meanings.retain(|meaning| meaning.strong_count() > 0);
                meanings.is_empty()
            });
        if remove_coordinate {
            state.meanings.remove(&self.coordinate);
        }
    }
}

struct WorthQueryObservedSourceMeaningRegistryState {
    next_ordinal: u64,
    meanings:
        BTreeMap<WorthQueryObservedSourceCoordinate, Vec<Weak<WorthQueryObservedSourceMeaning>>>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryObservedSourceMeaningRegistry {
    runtime_authority: u64,
    state: Arc<Mutex<WorthQueryObservedSourceMeaningRegistryState>>,
}

impl WorthQueryObservedSourceMeaningRegistry {
    pub(in crate::domain_computation::primary_graph) fn new(runtime_authority: u64) -> Self {
        Self {
            runtime_authority,
            state: Arc::new(Mutex::new(WorthQueryObservedSourceMeaningRegistryState {
                next_ordinal: 1,
                meanings: BTreeMap::new(),
            })),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn intern(
        &self,
        query: &[u8; 32],
        parameters: &[u8; 32],
        footprint: WorthQueryObservedSourceFootprint,
        selection: &WorthQueryApplicationBasisSelectionIdentity,
    ) -> Option<Arc<WorthQueryObservedSourceMeaning>> {
        let occurrence = match selection {
            WorthQueryApplicationBasisSelectionIdentity::Relational => {
                WorthQueryObservedSourceOccurrence::Relational
            }
            WorthQueryApplicationBasisSelectionIdentity::Product(product) => {
                WorthQueryObservedSourceOccurrence::Product(product.lifecycle_incarnation())
            }
        };
        let coordinate = WorthQueryObservedSourceCoordinate {
            query: *query,
            parameters: *parameters,
            occurrence,
            root: footprint.root,
        };
        // Keep every upgraded non-match alive until after the registry guard
        // is released. A concurrent final owner may otherwise leave this
        // temporary as the last Arc, whose Drop would re-lock this mutex.
        let mut unmatched_meanings = Vec::new();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(meanings) = state.meanings.get_mut(&coordinate) {
            meanings.retain(|meaning| meaning.strong_count() > 0);
            for existing in meanings.iter().filter_map(Weak::upgrade) {
                if existing.footprint == footprint {
                    drop(state);
                    drop(unmatched_meanings);
                    return Some(existing);
                }
                unmatched_meanings.push(existing);
            }
        }
        let ordinal = state.next_ordinal;
        let Some(next_ordinal) = ordinal.checked_add(1) else {
            drop(state);
            drop(unmatched_meanings);
            return None;
        };
        state.next_ordinal = next_ordinal;
        let checkpoint_identity =
            crate::domain_computation::primary_graph::application_checkpoint_source_identity::checkpoint_source_identity(
                query,
                parameters,
                &footprint,
            );
        let meaning = Arc::new(WorthQueryObservedSourceMeaning {
            coordinate: coordinate.clone(),
            footprint,
            identity: source_identity(self.runtime_authority, ordinal),
            checkpoint_identity,
            registry: Arc::downgrade(&self.state),
        });
        state
            .meanings
            .entry(coordinate)
            .or_default()
            .push(Arc::downgrade(&meaning));
        drop(state);
        drop(unmatched_meanings);
        Some(meaning)
    }
}

fn source_identity(runtime_authority: u64, ordinal: u64) -> [u8; 32] {
    let mut identity = [0; 32];
    identity[..8].copy_from_slice(b"WQSRCE01");
    identity[8..16].copy_from_slice(&runtime_authority.to_be_bytes());
    identity[16..24].copy_from_slice(&ordinal.to_be_bytes());
    identity
}

/// Complete runtime coordinate of one observed source.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryObservedSourceEpoch {
    query: [u8; 32],
    parameters: [u8; 32],
    root: worth_relational::facade::identity::EntityId,
    occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    observation_generation: u64,
    meaning: Arc<WorthQueryObservedSourceMeaning>,
}

impl WorthQueryObservedSourceEpoch {
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn new(
        query: [u8; 32],
        parameters: [u8; 32],
        root: worth_relational::facade::identity::EntityId,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        observation_generation: u64,
        identity: [u8; 32],
    ) -> Self {
        let footprint = WorthQueryObservedSourceFootprint {
            root,
            complete: true,
            entities: vec![root],
            aspects: Vec::new(),
            adjacencies: Vec::new(),
        };
        let checkpoint_identity =
            crate::domain_computation::primary_graph::application_checkpoint_source_identity::checkpoint_source_identity(
                &query,
                &parameters,
                &footprint,
            );
        Self {
            query,
            parameters,
            root,
            occurrence,
            observation_generation,
            meaning: Arc::new(WorthQueryObservedSourceMeaning {
                coordinate: WorthQueryObservedSourceCoordinate {
                    query,
                    parameters,
                    occurrence: WorthQueryObservedSourceOccurrence::Product(occurrence),
                    root,
                },
                footprint,
                identity,
                checkpoint_identity,
                registry: Weak::new(),
            }),
        }
    }

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
        Some(Self {
            query: *query,
            parameters: *parameters,
            root,
            occurrence: product.lifecycle_incarnation(),
            observation_generation: product.reference_generation().get(),
            meaning,
        })
    }

    pub(in crate::domain_computation::primary_graph) fn same_occurrence(
        &self,
        other: &Self,
    ) -> bool {
        self.query == other.query
            && self.parameters == other.parameters
            && self.root == other.root
            && self.occurrence == other.occurrence
    }

    pub(in crate::domain_computation::primary_graph) fn same_semantic_source(
        &self,
        other: &Self,
    ) -> bool {
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

    pub(in crate::domain_computation::primary_graph) const fn observation_generation(&self) -> u64 {
        self.observation_generation
    }

    pub(in crate::domain_computation::primary_graph) fn checkpoint_identity(&self) -> [u8; 32] {
        self.meaning.checkpoint_identity()
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) const fn checkpoint_occurrence(
        &self,
    ) -> worth_runtime_world::facade::ProductBranchIncarnation {
        self.occurrence
    }

    fn ordering_coordinates(
        &self,
    ) -> (
        [u8; 32],
        [u8; 32],
        worth_relational::facade::identity::EntityId,
        worth_runtime_world::facade::ProductBranchIncarnation,
        u64,
        [u8; 32],
    ) {
        (
            self.query,
            self.parameters,
            self.root,
            self.occurrence,
            self.observation_generation,
            *self.meaning.identity(),
        )
    }
}

impl PartialEq for WorthQueryObservedSourceEpoch {
    fn eq(&self, other: &Self) -> bool {
        self.ordering_coordinates() == other.ordering_coordinates()
    }
}

impl Eq for WorthQueryObservedSourceEpoch {}

impl PartialOrd for WorthQueryObservedSourceEpoch {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for WorthQueryObservedSourceEpoch {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ordering_coordinates()
            .cmp(&other.ordering_coordinates())
    }
}

impl Hash for WorthQueryObservedSourceEpoch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.query.hash(state);
        self.parameters.hash(state);
        self.root.hash(state);
        self.occurrence.hash(state);
        self.observation_generation.hash(state);
        self.meaning.identity().hash(state);
    }
}

#[cfg(test)]
mod tests;
