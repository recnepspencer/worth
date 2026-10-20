//! Measurements borrow this reader's original installed layout; no graph read.

use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::primary_graph::application_attempt::candidate_retained_representation as representation;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};

impl<Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn candidate_field_promotion_bytes(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Result<usize, Denial> {
        let layout = self.reader.layout;
        let locator = layout
            .field_locator(entity, aspect, field)
            .ok_or_else(|| Denial::new(Kind::UndeclaredEffect, field))?;
        let contract = layout
            .aspect_contract(entity, locator.aspect().aspect_key())
            .ok_or_else(|| Denial::new(Kind::UndeclaredEffect, field))?;
        Ok(representation::contract_width(
            &worth_foundational::facade::PortableAspectContractBasis::from_contract(contract),
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn candidate_field_bytes(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
        value_bytes: usize,
        existing_write: bool,
    ) -> Result<usize, Denial> {
        let layout = self.reader.layout;
        let locator = layout
            .field_locator(entity, aspect, field)
            .ok_or_else(|| Denial::new(Kind::UndeclaredEffect, field))?;
        let entry = if existing_write && layout.field_is_optional(entity, locator) {
            let contract = layout
                .aspect_contract(entity, locator.aspect().aspect_key())
                .ok_or_else(|| Denial::new(Kind::UndeclaredEffect, field))?;
            representation::optional_field_entry_bytes(
                locator,
                &worth_foundational::facade::PortableAspectContractBasis::from_contract(contract),
                value_bytes,
                true,
            )
        } else {
            representation::field_entry_bytes(locator, value_bytes, true)
        };
        entry.ok_or_else(|| Denial::new(Kind::CandidateReservationExceeded, field))
    }
}
