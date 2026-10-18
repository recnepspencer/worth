use serde::{Deserialize, Serialize};

use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;

use super::super::locality::{node_sort_key, PartitionScopeSet};
use super::super::SummaryForm;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum FrontierSeedCause {
    #[default]
    DirtySource,
    StructuralDelta,
    BatchRevalidation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum FrontierEntryClassification {
    DirectDirty,
    #[default]
    MaybeStale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, PartialOrd, Ord)]
pub enum FrontierInclusionBasis {
    #[default]
    DirectSubscriptionMatch,
    PartitionScopeOverlap,
    DetailScopeOverlap,
    TransitiveReachability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FrontierValidationDecision {
    #[default]
    ReachableCycleCheck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvalidationSeed {
    pub source_node: NodeId,
    pub aspect: Aspect,
    pub changed_scopes: PartitionScopeSet,
    pub cause: FrontierSeedCause,
}

impl InvalidationSeed {
    pub fn new(
        source_node: NodeId,
        aspect: Aspect,
        changed_scopes: impl Into<PartitionScopeSet>,
        cause: FrontierSeedCause,
    ) -> Self {
        Self {
            source_node,
            aspect,
            changed_scopes: changed_scopes.into(),
            cause,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InvalidationSeedBatch {
    pub seeds: Vec<InvalidationSeed>,
}

impl InvalidationSeedBatch {
    pub fn new(seeds: impl IntoIterator<Item = InvalidationSeed>) -> Self {
        let mut seeds = seeds.into_iter().collect::<Vec<_>>();
        if seeds.len() > 1 {
            seeds.sort_unstable_by_key(|seed| {
                (
                    seed.aspect.index(),
                    node_sort_key(&seed.source_node),
                    seed.changed_scopes.as_slice().to_vec(),
                    seed.cause,
                )
            });
            let mut merged = Vec::<InvalidationSeed>::with_capacity(seeds.len());
            for seed in seeds {
                if let Some(previous) = merged.last_mut()
                    && previous.source_node == seed.source_node
                    && previous.aspect == seed.aspect
                    && previous.cause == seed.cause
                {
                    let mut scopes = previous.changed_scopes.as_slice().to_vec();
                    scopes.extend_from_slice(seed.changed_scopes.as_slice());
                    previous.changed_scopes = PartitionScopeSet::new(scopes);
                    continue;
                }
                merged.push(seed);
            }
            seeds = merged;
        }
        Self { seeds }
    }

    pub fn as_slice(&self) -> &[InvalidationSeed] {
        &self.seeds
    }

    pub fn is_empty(&self) -> bool {
        self.seeds.is_empty()
    }
}

impl SummaryForm for InvalidationSeedBatch {}
