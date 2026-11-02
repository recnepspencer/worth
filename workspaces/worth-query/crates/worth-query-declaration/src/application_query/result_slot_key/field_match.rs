//! Borrowed equality uses the same complete contract as an owned field key.
use super::*;

#[cfg(test)]
mod tests;

impl ApplicationQueryResultSlotKey {
    pub(in crate::application_query) fn matches_field(
        &self,
        query: WorthQueryPortableTypeIdentity,
        slot: WorthQueryPortableTypeIdentity,
        requested: ApplicationQueryResultFieldSlotContract<'_>,
    ) -> bool {
        if self.query != query || self.slot != slot {
            return false;
        }
        match &self.contract {
            ApplicationQueryResultSlotContract::Field {
                entity,
                aspect,
                field,
                output_name,
                scalar_family,
                value_type,
                presence,
            } => {
                entity == requested.entity
                    && aspect == requested.aspect
                    && field == requested.field
                    && output_name == requested.output_name
                    && *scalar_family == requested.scalar_family
                    && *value_type == requested.value_type
                    && *presence == requested.presence
            }
            _ => false,
        }
    }
}
