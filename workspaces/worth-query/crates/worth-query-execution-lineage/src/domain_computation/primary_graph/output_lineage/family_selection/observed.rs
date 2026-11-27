//! A `current_output` family selection as a verifiable fact (public #429).
//!
//! The read observes a set: the verified-current members of the producer's
//! output family at the reader's coordinate. The fact retains that set and the
//! sources it was selected from, so a later head change can be keyed and a
//! certification can re-resolve the family. Marks never certify it.

use std::collections::BTreeSet;
use std::fmt;
use std::sync::{Arc, Weak};

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::EntityId;

use super::super::shared_backing::{SharedOutputLineage, WeakOutputLineage};
use super::super::SemanticSource;
use crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding;
use crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::primary_graph::SourceInvalidationOwner;
use crate::equality_type_id::EqualityTypeId as TypeId;

/// The family the read selected from and the entity kind it accepted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FamilySelectionProducer {
    pub runtime_authority: u64,
    pub schema: ApplicationSchemaBindingIdentity,
    pub scope: WorthQueryOperationScopeEntityBinding,
    pub family: &'static str,
    pub entity_type: TypeId,
    pub entity_identifier: &'static str,
}

/// The lineage, invalidation owner and layout of the reader that recorded
/// the fact. Weak, so a retained fact never keeps its runtime alive.
#[derive(Clone)]
struct FamilySelectionPort {
    lineage: WeakOutputLineage,
    owner: Weak<SourceInvalidationOwner>,
    layout: Weak<WorthQueryPrimaryGraphLayout>,
}

/// The port's owners while a certification re-resolves the family.
pub struct LiveFamilySelectionPort {
    pub lineage: SharedOutputLineage,
    pub owner: Arc<SourceInvalidationOwner>,
    pub layout: Arc<WorthQueryPrimaryGraphLayout>,
}

pub struct ObservedFamilySelection {
    pub producer: FamilySelectionProducer,
    /// Every installed source of the family, published or not.
    pub sources: Arc<[SemanticSource]>,
    /// The verified-current committed members the read selected, sorted.
    pub outcome: Arc<[EntityId]>,
    /// The members the applied overlay supplied, sorted.
    pub applied: Arc<[EntityId]>,
    /// The partitions the applied overlay masked.
    pub replaced: Arc<[(Option<TypeId>, [u8; 32])]>,
    port: FamilySelectionPort,
}

fn sorted(entities: impl IntoIterator<Item = EntityId>) -> Arc<[EntityId]> {
    entities.into_iter().collect::<BTreeSet<_>>().into_iter().collect()
}

impl ObservedFamilySelection {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        producer: FamilySelectionProducer,
        sources: Vec<SemanticSource>,
        outcome: impl IntoIterator<Item = EntityId>,
        applied: impl IntoIterator<Item = EntityId>,
        replaced: &[(Option<TypeId>, [u8; 32])],
        lineage: &SharedOutputLineage,
        owner: &Arc<SourceInvalidationOwner>,
        layout: &Arc<WorthQueryPrimaryGraphLayout>,
    ) -> Self {
        Self {
            producer,
            sources: sources.into(),
            outcome: sorted(outcome),
            applied: sorted(applied),
            replaced: replaced.into(),
            port: FamilySelectionPort {
                lineage: lineage.downgrade(),
                owner: Arc::downgrade(owner),
                layout: Arc::downgrade(layout),
            },
        }
    }

    /// The reader's owners, or `None` once any of them is gone.
    pub fn live_port(&self) -> Option<LiveFamilySelectionPort> {
        Some(LiveFamilySelectionPort {
            lineage: self.port.lineage.upgrade()?,
            owner: self.port.owner.upgrade()?,
            layout: self.port.layout.upgrade()?,
        })
    }

    /// Whether a re-resolution that selected `fresh`, plus the `extra`
    /// members an overlay supplied, selects what the read selected.
    pub fn selects_the_same(
        &self,
        extra: &[EntityId],
        fresh: impl IntoIterator<Item = EntityId>,
    ) -> bool {
        let recorded: BTreeSet<_> = self.applied.iter().chain(self.outcome.iter()).copied().collect();
        let resolved: BTreeSet<_> = extra.iter().copied().chain(fresh).collect();
        recorded == resolved
    }
}

impl fmt::Debug for ObservedFamilySelection {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output
            .debug_struct("ObservedFamilySelection")
            .field("producer", &self.producer)
            .field("outcome", &self.outcome)
            .field("applied", &self.applied)
            .finish_non_exhaustive()
    }
}

impl PartialEq for ObservedFamilySelection {
    fn eq(&self, other: &Self) -> bool {
        self.producer == other.producer
            && self.sources == other.sources
            && self.outcome == other.outcome
            && self.applied == other.applied
            && self.replaced == other.replaced
    }
}

impl Eq for ObservedFamilySelection {}
