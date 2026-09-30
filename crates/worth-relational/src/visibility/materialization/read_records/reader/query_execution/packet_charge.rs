use std::mem::size_of;

use worth_execution::ChargedBytes;

use super::super::query_packetization::PacketizedQueryWork;
use crate::identity::data::{EntityId, KindId};
use crate::storage::data::AuthoritativeFieldComparisonKey;
use crate::transactions::data::RecordRef;

impl ChargedBytes for PacketizedQueryWork {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::ExplicitTargets(targets) => vector_inline_bytes::<RecordRef>(targets.capacity()),
            Self::EntityKindScan { .. } | Self::RelationKindScan { .. } => 0,
            Self::EntityFieldEquals {
                field_locator,
                value,
                ..
            }
            | Self::RelationFieldEquals {
                field_locator,
                value,
                ..
            } => owned_bytes(field_locator.owned_allocation_capacity_bytes())
                .saturating_add(value.owned_allocation_capacity_bytes()),
            Self::EntityFieldAnyOf {
                field_locator,
                values,
                ..
            }
            | Self::RelationFieldAnyOf {
                field_locator,
                values,
                ..
            } => owned_bytes(field_locator.owned_allocation_capacity_bytes())
                .saturating_add(comparison_values_bytes(values)),
            Self::AspectFilteredEntities { aspect_filter, .. }
            | Self::AspectFilteredRelations { aspect_filter, .. } => {
                owned_bytes(aspect_filter.owned_allocation_capacity_bytes())
            }
            Self::OutgoingNeighborhood {
                seeds,
                relation_kind_scope,
            }
            | Self::IncomingNeighborhood {
                seeds,
                relation_kind_scope,
            }
            | Self::ConnectivityTraversal {
                seeds,
                relation_kind_scope,
                ..
            } => vector_inline_bytes::<EntityId>(seeds.capacity()).saturating_add(
                relation_kind_scope
                    .as_ref()
                    .map_or(0, |kinds| vector_inline_bytes::<KindId>(kinds.capacity())),
            ),
        }
    }
}

fn comparison_values_bytes(values: &Vec<AuthoritativeFieldComparisonKey>) -> u64 {
    values.iter().fold(
        vector_inline_bytes::<AuthoritativeFieldComparisonKey>(values.capacity()),
        |total, value| total.saturating_add(value.owned_allocation_capacity_bytes()),
    )
}

fn vector_inline_bytes<T>(capacity: usize) -> u64 {
    owned_bytes(capacity.saturating_mul(size_of::<T>()))
}

fn owned_bytes(bytes: usize) -> u64 {
    u64::try_from(bytes).unwrap_or(u64::MAX)
}
