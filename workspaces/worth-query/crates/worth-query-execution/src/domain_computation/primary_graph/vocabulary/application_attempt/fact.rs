//! Moved whole from `domain_computation::primary_graph::application_attempt::fact`: the items below and every impl they own.
use crate::domain_computation::primary_graph::vocabulary::application_attempt::managed_source_read::ManagedSourceReadKey;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::source_publication_read::SourcePublicationRead;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::source_read_authorization::SourceReadAuthorizationKey;
use worth_execution::ChargedBytes;
use worth_foundational::facade::AspectFieldLocator;
use worth_relational::facade::identity::EntityId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryApplicationAdjacencyDirection {
    Outgoing,
    Incoming,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryApplicationFactKey {
    StructuralWalk(crate::domain_computation::primary_graph::vocabulary::structural_walk::key::StructuralWalkKey),
    Aggregate(crate::domain_computation::primary_graph::vocabulary::aggregate_observation::key::AggregateKey),
    ComponentSelection(crate::domain_computation::primary_graph::vocabulary::component_selection::key::ComponentSelectionKey),
    SourcePublication(SourcePublicationRead),
    ManagedIndexSource(ManagedSourceReadKey),
    SourceReadAuthorization(SourceReadAuthorizationKey),
    Entity {
        entity: String,
        entity_id: EntityId,
    },
    Field {
        entity: String,
        entity_id: EntityId,
        locator: AspectFieldLocator,
    },
    AspectPresence {
        entity: String,
        entity_id: EntityId,
        locator: AspectFieldLocator,
    },
    Relation {
        relation: String,
        from: EntityId,
        to: EntityId,
    },
    Adjacency {
        relation: String,
        anchor: EntityId,
        direction: WorthQueryApplicationAdjacencyDirection,
        maximum_work_units: usize,
    },
}

impl ChargedBytes for WorthQueryApplicationFactKey {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            // The native read retains its own actual backing and custody.
            Self::ComponentSelection(_)
            | Self::ManagedIndexSource(_)
            | Self::SourceReadAuthorization(_)
            | Self::SourcePublication(_)
            | Self::StructuralWalk(_)
            | Self::Aggregate(_) => 0,
            Self::Entity { entity, .. } => bytes(entity.capacity()),
            Self::AspectPresence {
                entity, locator, ..
            }
            | Self::Field {
                entity, locator, ..
            } => bytes(entity.capacity())
                .saturating_add(bytes(locator.owned_allocation_capacity_bytes())),
            Self::Relation { relation, .. } | Self::Adjacency { relation, .. } => {
                bytes(relation.capacity())
            }
        }
    }
}

pub(in crate::domain_computation::primary_graph) fn bytes(count: usize) -> u64 {
    u64::try_from(count).unwrap_or(u64::MAX)
}
