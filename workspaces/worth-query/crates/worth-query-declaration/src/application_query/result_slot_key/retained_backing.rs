//! Physical backing belongs to the exact slot contract, including its binding.
use super::{ApplicationQueryResultSlotContract as Contract, ApplicationQueryResultSlotKey};

impl ApplicationQueryResultSlotKey {
    /// Heap backing retained by this slot's complete immutable interpretation.
    /// Inline enum fields and borrowed portable identities allocate nothing.
    pub fn retained_heap_bytes(&self) -> Option<usize> {
        let contract = match &self.contract {
            Contract::Resource { .. } | Contract::HistoryMetadata { .. } => 0,
            Contract::Components {
                set,
                member,
                output_name,
                ..
            } => string_backing([set, member, output_name])?,
            Contract::Orbit {
                relation,
                from,
                to,
                output_name,
            }
            | Contract::Relation {
                relation,
                from,
                to,
                output_name,
                ..
            } => string_backing([relation, from, to, output_name])?,
            Contract::Field {
                entity,
                aspect,
                field,
                output_name,
                value_type,
                ..
            } => string_backing([entity, aspect, field, output_name])?
                .checked_add(value_type.retained_heap_bytes())?,
            Contract::InspectionMetadata { binding, .. } => binding.retained_heap_bytes()?,
        };
        self.query
            .retained_heap_bytes()
            .checked_add(self.slot.retained_heap_bytes())?
            .checked_add(contract)
    }
}

fn string_backing<const N: usize>(strings: [&String; N]) -> Option<usize> {
    strings
        .into_iter()
        .map(String::capacity)
        .try_fold(0usize, usize::checked_add)
}
