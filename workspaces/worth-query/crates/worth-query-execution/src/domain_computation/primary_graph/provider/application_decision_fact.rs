use super::super::WorthQueryApplicationAttemptDenial as SelfDenial;
use std::sync::Arc;

use crate::domain_computation::authorization::WorthQueryPrincipalCurrentnessDependency;

#[derive(Clone)]
pub(in crate::domain_computation) enum WorthQueryPrimaryGraphApplicationDecisionFact {
    Application {
        read_scope: worth_query_installation::facade::WorthQueryOperationGraphReadScope,
        fact: super::super::application_attempt::WorthQueryApplicationObservedFact,
    },
    ObservedSource {
        fact: super::super::application_attempt::WorthQueryApplicationObservedFact,
    },
    ApplicationObservedSource {
        read_scope: worth_query_installation::facade::WorthQueryOperationGraphReadScope,
        fact: super::super::application_attempt::WorthQueryApplicationObservedFact,
    },
    Principal(WorthQueryPrincipalCurrentnessDependency),
    Authorization {
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        locator: Arc<str>,
        decision: crate::domain_computation::authorization::WorthQueryAuthorizationDecisionFact,
    },
}

impl WorthQueryPrimaryGraphApplicationDecisionFact {
    pub(in crate::domain_computation) const fn application(
        read_scope: worth_query_installation::facade::WorthQueryOperationGraphReadScope,
        fact: super::super::application_attempt::WorthQueryApplicationObservedFact,
    ) -> Self {
        Self::Application { read_scope, fact }
    }

    pub(in crate::domain_computation) const fn principal(
        dependency: WorthQueryPrincipalCurrentnessDependency,
    ) -> Self {
        Self::Principal(dependency)
    }

    pub(in crate::domain_computation) const fn observed_source(
        fact: super::super::application_attempt::WorthQueryApplicationObservedFact,
    ) -> Self {
        Self::ObservedSource { fact }
    }

    pub(in crate::domain_computation) fn retain_observed_source_role(
        self,
        source: super::super::application_attempt::WorthQueryApplicationObservedFact,
    ) -> Result<Self, &'static str> {
        match self {
            Self::Application { read_scope, fact } if fact == source => {
                Ok(Self::ApplicationObservedSource { read_scope, fact })
            }
            Self::ObservedSource { fact } if fact == source => Ok(Self::ObservedSource { fact }),
            Self::ApplicationObservedSource { read_scope, fact } if fact == source => {
                Ok(Self::ApplicationObservedSource { read_scope, fact })
            }
            _ => Err("decision facts disagree at one structural locator"),
        }
    }

    pub(in crate::domain_computation) const fn is_application_decision(&self) -> bool {
        matches!(
            self,
            Self::Application { .. }
                | Self::ObservedSource { .. }
                | Self::ApplicationObservedSource { .. }
        )
    }

    pub(in crate::domain_computation) fn authorization(
        requirement_ordinal: usize,
        dependency: crate::domain_computation::authorization::WorthQueryAuthorizationDecisionFact,
    ) -> Self {
        let session = dependency.session_identity();
        let locator = format!("application-authorization:{requirement_ordinal}");
        let fact = Self::Authorization {
            session,
            locator: Arc::from(locator),
            decision: dependency,
        };
        debug_assert_eq!(fact.session_identity(), Some(session));
        fact
    }

    pub(in crate::domain_computation) fn session_identity(
        &self,
    ) -> Option<crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity>
    {
        match self {
            Self::Principal(dependency) => Some(dependency.session_identity()),
            Self::Authorization { session, .. } => Some(*session),
            Self::Application { .. } => None,
            Self::ObservedSource { .. } => None,
            Self::ApplicationObservedSource { .. } => None,
        }
    }

    pub(in crate::domain_computation) fn locator_identity(&self) -> String {
        match self {
            Self::Application { fact, .. } => fact.dependency_locator_identity(),
            Self::ObservedSource { fact } => fact.dependency_locator_identity(),
            Self::ApplicationObservedSource { fact, .. } => fact.dependency_locator_identity(),
            Self::Principal(_) => "application-principal-currentness".to_string(),
            Self::Authorization { locator, .. } => locator.to_string(),
        }
    }

    pub(in crate::domain_computation::primary_graph) fn observed_source_fact(
        &self,
    ) -> Option<&super::super::application_attempt::WorthQueryApplicationObservedFact> {
        match self {
            Self::ObservedSource { fact } | Self::ApplicationObservedSource { fact, .. } => {
                Some(fact)
            }
            _ => None,
        }
    }

    pub(in crate::domain_computation) fn remains_equal_in(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<bool, super::super::WorthQueryApplicationAttemptDenial> {
        #[cfg(test)]
        let _scope = super::super::application_attempt::consult_observation::ReadConsultScopeForTest::enter_for_test(
            super::super::application_attempt::consult_observation::Door::Equality,
        );
        let equal = match self {
            Self::Application { fact, .. } => {
                fact.remains_equal_in(runtime, execution, snapshot)?
            }
            Self::ObservedSource { fact } => fact.remains_equal_in(runtime, execution, snapshot)?,
            Self::ApplicationObservedSource { fact, .. } => {
                fact.remains_equal_in(runtime, execution, snapshot)?
            }
            Self::Principal(dependency) => dependency
                .remains_current_in(execution, runtime, snapshot)
                .map_err(|stop| SelfDenial::principal_read_stopped(stop, "principal comparison"))?,
            Self::Authorization { decision, .. } => decision
                .remains_equal_in(execution, runtime, snapshot)
                .map_err(|cause| {
                    SelfDenial::relational_read_stopped(cause, "authorization comparison")
                })?,
        };
        Ok(equal)
    }
}
