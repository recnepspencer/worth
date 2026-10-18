//! Atomic binding of authorization observations to provider read-set facts.

use std::{collections::BTreeMap, sync::Arc};

use super::{WorthQueryAuthorizationDecisionFact, WorthQueryPrincipalCurrentnessDependency};
use crate::domain_computation::primary_graph::{
    CompletedApplicationFacts, WorthQueryPrimaryGraphApplicationDecisionFact,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;
use crate::domain_computation::{WorthQueryDecisionFactLocator, WorthQueryDecisionFactRequest};
use worth_query_installation::facade::{
    APPLICATION_AUTHORIZATION_FACT_FAMILY, APPLICATION_DECISION_FACT_FAMILY,
};

pub(in crate::domain_computation) struct WorthQueryProviderAuthorizationDecisionFacts {
    principal: WorthQueryPrincipalCurrentnessDependency,
    decisions: Vec<WorthQueryAuthorizationDecisionFact>,
}

pub(in crate::domain_computation) struct WorthQueryProviderDecisionFactBinding {
    facts: Arc<BTreeMap<String, WorthQueryPrimaryGraphApplicationDecisionFact>>,
    requests: Vec<WorthQueryDecisionFactRequest>,
    retained_authorization_fact_count: usize,
    session: WorthQueryGraphWorkSessionIdentity,
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
        application: CompletedApplicationFacts,
    ) -> Result<WorthQueryProviderDecisionFactBinding, &'static str> {
        let retained_authorization_fact_count = 1usize.saturating_add(self.decisions.len());
        let session = self.principal.session_identity();
        if self
            .decisions
            .iter()
            .any(|decision| decision.session_identity() != session)
        {
            return Err("authorization facts belong to different graph-work sessions");
        }
        let mut facts = application.into_decisions()?;
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
            facts: Arc::new(facts),
            requests,
            retained_authorization_fact_count,
            session,
        })
    }
}

impl WorthQueryProviderDecisionFactBinding {
    pub(in crate::domain_computation) fn validate_session(
        &self,
        graph_work_session: &WorthQueryGraphWorkSessionIdentity,
        expected_authorization_fact_count: usize,
    ) -> Result<(), &'static str> {
        // Construction checked EVERY freshly issued authorization observation.
        // The immutable application projection has no session observations.
        if self.session != *graph_work_session
            || self.retained_authorization_fact_count != expected_authorization_fact_count
        {
            return Err("provider decision facts do not close over the graph-work session");
        }
        Ok(())
    }

    pub(in crate::domain_computation) fn facts(
        &self,
    ) -> &BTreeMap<String, WorthQueryPrimaryGraphApplicationDecisionFact> {
        &self.facts
    }

    pub(in crate::domain_computation) fn shared_facts(
        &self,
    ) -> Arc<BTreeMap<String, WorthQueryPrimaryGraphApplicationDecisionFact>> {
        Arc::clone(&self.facts)
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
