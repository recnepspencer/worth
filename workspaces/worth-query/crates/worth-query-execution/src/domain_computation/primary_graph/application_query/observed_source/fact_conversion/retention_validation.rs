//! Borrowed validation required even when recorded resolution skips payload copies.
use super::*;
use std::collections::BTreeMap;

impl<Query> WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation::primary_graph::application_query::observed_source) fn validate_fact_retention(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        expected: &str,
        control: SourceFactExpansionControl<'_, '_>,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        control.admit(self, expected)?;
        self.validate_completeness(expected)?;
        source_fact_count(self, expected)?;
        let footprint = self.source_meaning.footprint();
        footprint.validate_selections(layout, &|subject| control.poll(subject))?;
        // Temporary borrowed-key indexes remain outside payload custody. They
        // validate the same duplicate body rule without constructing a Fact or
        // cloning endpoints, field locators, or canonical identity material.
        let mut fields = BTreeMap::new();
        for field in footprint.fact_sources().flat_map(|facts| facts.aspects) {
            control.poll(expected)?;
            let contract = layout
                .aspect_contract(&field.entity_name, &field.aspect)
                .ok_or_else(|| source_contract_denial(field))?;
            ValidatedSourceField::resolve(field, contract)?;
            if let Some(previous) = fields.insert(
                (field.entity, &field.aspect, &field.field),
                field.native_revision,
            ) && previous != field.native_revision
            {
                return Err(changed(expected));
            }
        }
        let mut adjacencies = BTreeMap::new();
        for adjacency in footprint.fact_sources().flat_map(|facts| facts.adjacencies) {
            control.poll(expected)?;
            let key = (
                adjacency.anchor,
                adjacency.relation_kind,
                crate::domain_computation::primary_graph::source_facts::adjacency_direction_rank(
                    adjacency.direction,
                ),
            );
            if let Some(previous) = adjacencies.insert(key, adjacency.native_revision)
                && previous != adjacency.native_revision
            {
                return Err(changed(expected));
            }
        }
        control.poll(expected)
    }
}
fn changed(expected: &str) -> WorthQuerySourceExpectationDenial {
    WorthQuerySourceExpectationDenial::new(
        WorthQuerySourceExpectationDenialKind::SourceChanged,
        expected,
    )
}
