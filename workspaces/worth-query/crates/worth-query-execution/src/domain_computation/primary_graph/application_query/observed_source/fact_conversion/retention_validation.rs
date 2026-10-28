//! Borrowed validation required even when recorded resolution skips payload copies.
use super::*;
use std::collections::BTreeMap;

impl<Query> WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation::primary_graph::application_query::observed_source) fn validate_fact_retention(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        expected: &str,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
    ) -> Result<(), WorthQuerySourceExpectationDenial> {
        self.require_supported_fact_expansion(expected)?;
        self.validate_completeness(expected)?;
        source_fact_count(self, expected)?;
        let footprint = self.source_meaning.footprint();
        footprint.validate_selections(layout, request)?;
        // Temporary borrowed-key indexes remain outside payload custody. They
        // validate the same duplicate body rule without constructing a Fact or
        // cloning endpoints, field locators, or canonical identity material.
        let mut fields = BTreeMap::new();
        for field in footprint.fact_sources().flat_map(|facts| facts.aspects) {
            check_request(request, expected)?;
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
            check_request(request, expected)?;
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
        check_request(request, expected)
    }
}
fn changed(expected: &str) -> WorthQuerySourceExpectationDenial {
    WorthQuerySourceExpectationDenial::new(
        WorthQuerySourceExpectationDenialKind::SourceChanged,
        expected,
    )
}
fn check_request(
    request: Option<
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    >,
    expected: &str,
) -> Result<(), WorthQuerySourceExpectationDenial> {
    if let Some(stop) = request.and_then(|request| request.interruption()) {
        return Err(WorthQuerySourceExpectationDenial::source_retention_denied(expected, crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::RequestInterruption(stop)));
    }
    Ok(())
}
