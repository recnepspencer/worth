//! Atomic binding of authorization observations to provider read-set facts.

use std::sync::Arc;

use super::{WorthQueryAuthorizationDecisionFact, WorthQueryPrincipalCurrentnessDependency};
use crate::domain_computation::primary_graph::{
    ApplicationFactTable, CompletedApplicationFacts, WorthQueryPrimaryGraphApplicationDecisionFact,
};
use crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity;

pub(in crate::domain_computation) struct WorthQueryProviderAuthorizationDecisionFacts {
    principal: WorthQueryPrincipalCurrentnessDependency,
    decisions: Vec<WorthQueryAuthorizationDecisionFact>,
}

pub(in crate::domain_computation) struct WorthQueryProviderDecisionFactBinding {
    facts: Arc<ApplicationFactTable>,
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

    pub(in crate::domain_computation) fn fact_count(&self) -> usize {
        1 + self.decisions.len()
    }

    pub(in crate::domain_computation) fn bind_application_facts(
        self,
        application: CompletedApplicationFacts,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        WorthQueryProviderDecisionFactBinding,
        crate::domain_computation::WorthQueryDecisionReadSetFailure,
    > {
        let invalid = |detail| {
            crate::domain_computation::WorthQueryDecisionReadSetFailure::new(
                crate::domain_computation::WorthQueryDecisionReadSetDenialKind::ProviderRejected,
                detail,
            )
        };
        let retained_authorization_fact_count = 1usize.saturating_add(self.decisions.len());
        let session = self.principal.session_identity();
        if self
            .decisions
            .iter()
            .any(|decision| decision.session_identity() != session)
        {
            return Err(invalid(
                "authorization facts belong to different graph-work sessions",
            ));
        }
        let mut facts = application.into_table(execution)?;
        facts
            .insert_fresh(WorthQueryPrimaryGraphApplicationDecisionFact::principal(
                self.principal,
            ))
            .map_err(invalid)?;
        for (ordinal, observation) in self.decisions.into_iter().enumerate() {
            facts
                .insert_fresh(
                    WorthQueryPrimaryGraphApplicationDecisionFact::authorization(
                        ordinal,
                        observation,
                    ),
                )
                .map_err(invalid)?;
        }
        Ok(WorthQueryProviderDecisionFactBinding {
            facts: Arc::new(facts),
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

    pub(in crate::domain_computation) fn facts(&self) -> &ApplicationFactTable {
        &self.facts
    }

    pub(in crate::domain_computation) fn shared_facts(&self) -> Arc<ApplicationFactTable> {
        Arc::clone(&self.facts)
    }

    pub(in crate::domain_computation) fn decision_fact_count(&self) -> usize {
        self.facts.len()
    }
}
