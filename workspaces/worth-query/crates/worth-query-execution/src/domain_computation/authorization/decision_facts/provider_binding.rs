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
    ) -> Result<WorthQueryProviderDecisionFactBinding, &'static str> {
        if installed_read_scopes.len() > application.len() {
            return Err("installed read scopes exceed retained application facts");
        }
        let installed_count = installed_read_scopes.len();
        let retained_authorization_fact_count = 1usize.saturating_add(self.decisions.len());
        let mut application = application;
        let source_facts = application.split_off(installed_count);
        let mut facts = installed_read_scopes
            .into_iter()
            .zip(application)
            .map(|(read_scope, fact)| {
                WorthQueryPrimaryGraphApplicationDecisionFact::application(read_scope, fact)
            })
            .collect::<Vec<_>>();
        let mut fact_indices = facts
            .iter()
            .enumerate()
            .map(|(index, fact)| (fact.locator_identity(), index))
            .collect::<BTreeMap<_, _>>();
        if fact_indices.len() != facts.len() {
            return Err("application decision facts contain duplicate structural locators");
        }
        for source in source_facts {
            let locator = source.locator_identity();
            if let Some(index) = fact_indices.get(&locator).copied() {
                facts[index] = facts[index].clone().retain_observed_source_role(source)?;
            } else {
                fact_indices.insert(locator, facts.len());
                facts.push(WorthQueryPrimaryGraphApplicationDecisionFact::observed_source(source));
            }
        }
        facts.push(WorthQueryPrimaryGraphApplicationDecisionFact::principal(
            self.principal,
        ));
        facts.extend(
            self.decisions
                .into_iter()
                .enumerate()
                .map(|(ordinal, observation)| {
                    WorthQueryPrimaryGraphApplicationDecisionFact::authorization(
                        ordinal,
                        observation,
                    )
                }),
        );
        let requests = bind_requests(&facts)?;
        let decision_fact_count = facts.len();
        let facts = facts
            .into_iter()
            .map(|fact| (fact.locator_identity(), fact))
            .collect::<BTreeMap<_, _>>();
        if facts.len() != decision_fact_count {
            return Err("decision facts contain duplicate structural locators");
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
) -> Result<Vec<WorthQueryDecisionFactRequest>, &'static str> {
    facts
        .iter()
        .map(|fact| {
            let family = if fact.is_application_decision() {
                APPLICATION_DECISION_FACT_FAMILY
            } else {
                APPLICATION_AUTHORIZATION_FACT_FAMILY
            };
            WorthQueryDecisionFactLocator::structural_proof(fact.locator_identity())
                .map_err(|_| "decision fact locator is invalid")
                .and_then(|locator| {
                    WorthQueryDecisionFactRequest::new(family, locator)
                        .map_err(|_| "decision fact request is invalid")
                })
        })
        .collect()
}
