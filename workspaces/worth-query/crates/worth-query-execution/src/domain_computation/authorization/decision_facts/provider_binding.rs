//! Atomic binding of authorization observations to provider read-set facts.

use std::collections::BTreeMap;

use super::{WorthQueryAuthorizationDecisionFact, WorthQueryPrincipalCurrentnessDependency};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationObservedFact, WorthQueryPrimaryGraphApplicationDecisionFact,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
use crate::domain_computation::{WorthQueryDecisionFactLocator, WorthQueryDecisionFactRequest};
use worth_query_installation::facade::{
    WorthQueryOperationGraphReadScope, APPLICATION_AUTHORIZATION_FACT_FAMILY,
    APPLICATION_DECISION_FACT_FAMILY,
};

pub(in crate::domain_computation) struct WorthQueryProviderAuthorizationDecisionFacts {
    principal: WorthQueryPrincipalCurrentnessDependency,
    decisions: Vec<WorthQueryAuthorizationDecisionFact>,
}

pub(in crate::domain_computation) struct WorthQueryProviderDecisionFactBinding {
    facts: BTreeMap<String, WorthQueryPrimaryGraphApplicationDecisionFact>,
    requests: Vec<WorthQueryDecisionFactRequest>,
    retained_authorization_fact_count: usize,
}

impl WorthQueryProviderAuthorizationDecisionFacts {
    pub(in crate::domain_computation::authorization) fn new(
        principal: WorthQueryPrincipalCurrentnessDependency,
        decisions: Vec<WorthQueryAuthorizationDecisionFact>,
    ) -> Self {
        Self {
            principal,
            decisions,
        }
    }

    pub(in crate::domain_computation) fn bind_application_facts(
        self,
        installed_read_scopes: Vec<WorthQueryOperationGraphReadScope>,
        application: Vec<WorthQueryApplicationObservedFact>,
    ) -> Result<WorthQueryProviderDecisionFactBinding, ()> {
        if installed_read_scopes.len() > application.len() {
            return Err(());
        }
        let application_count = application.len();
        let installed_count = installed_read_scopes.len();
        let retained_authorization_fact_count = 1usize.saturating_add(self.decisions.len());
        let mut application = application;
        let source_facts = application.split_off(installed_count);
        let facts = installed_read_scopes
            .into_iter()
            .zip(application)
            .map(|(read_scope, fact)| {
                WorthQueryPrimaryGraphApplicationDecisionFact::application(read_scope, fact)
            })
            .chain(
                source_facts
                    .into_iter()
                    .map(WorthQueryPrimaryGraphApplicationDecisionFact::observed_source),
            )
            .chain(std::iter::once(
                WorthQueryPrimaryGraphApplicationDecisionFact::principal(self.principal),
            ))
            .chain(
                self.decisions
                    .into_iter()
                    .enumerate()
                    .map(|(ordinal, observation)| {
                        WorthQueryPrimaryGraphApplicationDecisionFact::authorization(
                            ordinal,
                            observation,
                        )
                    }),
            )
            .collect::<Vec<_>>();
        let requests = bind_requests(&facts, application_count)?;
        let decision_fact_count = facts.len();
        let facts = facts
            .into_iter()
            .map(|fact| (fact.locator_identity(), fact))
            .collect::<BTreeMap<_, _>>();
        if facts.len() != decision_fact_count {
            return Err(());
        }
        Ok(WorthQueryProviderDecisionFactBinding {
            facts,
            requests,
            retained_authorization_fact_count,
        })
    }
}

impl WorthQueryProviderDecisionFactBinding {
    pub(in crate::domain_computation) fn validate_session(
        &self,
        graph_work_session: &WorthQueryGraphWorkSessionIdentity,
        expected_authorization_fact_count: usize,
    ) -> Result<(), &'static str> {
        let retained_facts = self
            .facts
            .values()
            .filter_map(WorthQueryPrimaryGraphApplicationDecisionFact::session_identity);
        if retained_facts
            .clone()
            .any(|session| session != *graph_work_session)
            || retained_facts.count() != expected_authorization_fact_count
            || self.retained_authorization_fact_count != expected_authorization_fact_count
        {
            return Err("provider decision facts do not close over the graph-work session");
        }
        Ok(())
    }

    pub(in crate::domain_computation) const fn facts(
        &self,
    ) -> &BTreeMap<String, WorthQueryPrimaryGraphApplicationDecisionFact> {
        &self.facts
    }

    pub(in crate::domain_computation) fn decision_fact_count(&self) -> usize {
        self.facts.len()
    }

    pub(in crate::domain_computation) fn take_read_requests(
        &mut self,
    ) -> Vec<WorthQueryDecisionFactRequest> {
        std::mem::take(&mut self.requests)
    }
}

fn bind_requests(
    facts: &[WorthQueryPrimaryGraphApplicationDecisionFact],
    application_count: usize,
) -> Result<Vec<WorthQueryDecisionFactRequest>, ()> {
    facts
        .iter()
        .enumerate()
        .map(|(index, fact)| {
            let family = if index < application_count {
                APPLICATION_DECISION_FACT_FAMILY
            } else {
                APPLICATION_AUTHORIZATION_FACT_FAMILY
            };
            WorthQueryDecisionFactLocator::structural_proof(fact.locator_identity())
                .map_err(|_| ())
                .and_then(|locator| {
                    WorthQueryDecisionFactRequest::new(family, locator).map_err(|_| ())
                })
        })
        .collect()
}
