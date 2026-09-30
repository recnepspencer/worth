use std::mem::size_of;

use worth_execution::ChargedBytes;

use crate::query::data::{
    CanonicalQueryResult, QueryWorkerFragment, TraversalEntityVisitKey, TraversalReductionBasis,
    TraversalRelationVisitKey,
};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};

impl ChargedBytes for EntityReadRecord {
    fn additional_charged_bytes(&self) -> u64 {
        let aspect_bytes = self.authoritative_aspect_state.as_ref().map_or(0, |state| {
            u64::try_from(state.owned_allocation_capacity_bytes()).unwrap_or(u64::MAX)
        });
        self.kind
            .kind_name
            .additional_charged_bytes()
            .saturating_add(self.kind.schema_id.0.additional_charged_bytes())
            .saturating_add(aspect_bytes)
    }
}

impl ChargedBytes for RelationReadRecord {
    fn additional_charged_bytes(&self) -> u64 {
        let aspect_bytes = self.authoritative_aspect_state.as_ref().map_or(0, |state| {
            u64::try_from(state.owned_allocation_capacity_bytes()).unwrap_or(u64::MAX)
        });
        self.kind
            .kind_name
            .additional_charged_bytes()
            .saturating_add(self.kind.schema_id.0.additional_charged_bytes())
            .saturating_add(aspect_bytes)
    }
}

impl ChargedBytes for TraversalReductionBasis {
    fn additional_charged_bytes(&self) -> u64 {
        allocation_bytes::<TraversalEntityVisitKey>(self.entity_visit_keys.capacity())
            .saturating_add(allocation_bytes::<TraversalRelationVisitKey>(
                self.relation_visit_keys.capacity(),
            ))
    }
}

impl ChargedBytes for TraversalEntityVisitKey {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl ChargedBytes for TraversalRelationVisitKey {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl ChargedBytes for QueryWorkerFragment {
    fn additional_charged_bytes(&self) -> u64 {
        self.entities
            .additional_charged_bytes()
            .saturating_add(self.relations.additional_charged_bytes())
            .saturating_add(self.traversal_basis.additional_charged_bytes())
    }
}

impl ChargedBytes for CanonicalQueryResult {
    fn additional_charged_bytes(&self) -> u64 {
        self.entities
            .additional_charged_bytes()
            .saturating_add(self.relations.additional_charged_bytes())
            .saturating_add(self.reduction_digest.additional_charged_bytes())
    }
}

fn allocation_bytes<T>(capacity: usize) -> u64 {
    capacity
        .checked_mul(size_of::<T>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .unwrap_or(u64::MAX)
}
