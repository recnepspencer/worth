//! Validate installed meaning before retaining original complete Native bodies.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::observed_source::{
        WorthQuerySourceExpectationDenial as Denial, WorthQuerySourceExpectationDenialKind as Kind,
    },
    WorthQueryPrimaryGraphLayout,
};

impl WorthQueryObservedRootSelection {
    pub(in crate::domain_computation::primary_graph::application_query) fn validate_walks(
        &self,
        layout: &WorthQueryPrimaryGraphLayout,
        request: Option<
            &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        >,
    ) -> Result<(), Denial> {
        if let Some(orbit) = &self.orbit {
            let query = orbit.aggregate.key();
            if !layout.relation(query.subject()).is_some_and(|relation| {
                relation.orbit
                    && Some(relation.kind) == query.orbit_relation()
                    && relation.from == query.entity_kind()
                    && relation.to == query.entity_kind()
            }) {
                return Err(Denial::new(Kind::SourceContractMismatch, query.subject()));
            }
        }
        if let Some(components) = &self.components {
            let expected = layout
                .component_set(components.meaning.set())
                .ok_or_else(|| {
                    Denial::new(Kind::SourceContractMismatch, components.meaning.set())
                })?;
            if expected.definition != components.body.layout().definition
                || expected.member.as_ref() != components.meaning.member()
                || !components.body.has_query_origin(&components.binding)
            {
                return Err(Denial::new(
                    Kind::SourceContractMismatch,
                    components.meaning.set(),
                ));
            }
        }
        for body in self.walks.iter().flat_map(|owned| owned.values()) {
            if let Some(stop) = request.and_then(|r| r.interruption()) {
                return Err(Denial::source_retention_denied(body.key().relation(), crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StoreDenial::RequestInterruption(stop)));
            }
            let query = body.key();
            if !layout.relation(query.relation()).is_some_and(|relation| {
                relation.kind == query.relation_kind()
                    && relation.from == query.entity_kind()
                    && relation.to == query.entity_kind()
            }) {
                return Err(Denial::new(Kind::SourceContractMismatch, query.relation()));
            }
        }
        Ok(())
    }
}
