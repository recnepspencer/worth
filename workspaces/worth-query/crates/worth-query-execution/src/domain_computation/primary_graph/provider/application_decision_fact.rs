pub(crate) mod application_facts;
pub(crate) mod application_table;
use super::super::WorthQueryApplicationAttemptDenial as SelfDenial;
use crate::domain_computation::{
    WorthQueryDecisionReadSetDenialKind as BindingKind,
    WorthQueryDecisionReadSetFailure as BindingFailure,
};
pub(in crate::domain_computation) use application_facts::{
    CompletedApplicationFacts, SharedObservedFact, SharedReadScope,
};
pub(in crate::domain_computation) use application_table::ApplicationFactTable;
use std::sync::Arc;
use worth_execution::ExecutionRequest;

use crate::domain_computation::authorization::WorthQueryPrincipalCurrentnessDependency;

#[derive(Clone)]
pub enum WorthQueryPrimaryGraphApplicationDecisionFact {
    Application {
        read_scope: SharedReadScope,
        fact: SharedObservedFact,
    },
    ObservedSource {
        fact: SharedObservedFact,
    },
    ApplicationObservedSource {
        read_scope: SharedReadScope,
        fact: SharedObservedFact,
    },
    Principal(WorthQueryPrincipalCurrentnessDependency),
    Authorization {
        session: crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity,
        locator: Arc<str>,
        decision: crate::domain_computation::authorization::WorthQueryAuthorizationDecisionFact,
    },
}

impl WorthQueryPrimaryGraphApplicationDecisionFact {
    pub const fn application(read_scope: SharedReadScope, fact: SharedObservedFact) -> Self {
        Self::Application { read_scope, fact }
    }

    pub(in crate::domain_computation) const fn principal(
        dependency: WorthQueryPrincipalCurrentnessDependency,
    ) -> Self {
        Self::Principal(dependency)
    }

    pub(in crate::domain_computation) const fn observed_source(fact: SharedObservedFact) -> Self {
        Self::ObservedSource { fact }
    }

    pub(in crate::domain_computation) fn retain_observed_source_role(
        self,
        source: SharedObservedFact,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, BindingFailure> {
        match self {
            Self::Application { read_scope, fact } => Ok(Self::ApplicationObservedSource {
                read_scope,
                fact: retained_observation(fact, source, execution)?,
            }),
            Self::ObservedSource { fact } => Ok(Self::ObservedSource {
                fact: retained_observation(fact, source, execution)?,
            }),
            Self::ApplicationObservedSource { read_scope, fact } => {
                Ok(Self::ApplicationObservedSource {
                    read_scope,
                    fact: retained_observation(fact, source, execution)?,
                })
            }
            _ => Err(binding_failure(DISAGREEMENT)),
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
    ) -> Option<crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity>
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
                Some(fact.as_ref())
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
            Self::Principal(dependency) => {
                use crate::domain_computation::vocabulary::primary_graph::kind::WorthQueryApplicationAttemptDenialKind as Attempt;
                use crate::domain_computation::vocabulary::authorization::denial::WorthQueryOperationAuthorizationDenialKind as Authorization;
                match dependency.remains_current_in(execution, runtime, snapshot) {
                    Ok(current) => current,
                    Err(kind) => {
                        let stop = match kind {
                            Authorization::Stopped(Ok(kind)) => {
                                Some(Attempt::ExecutionRequest(kind))
                            }
                            Authorization::Stopped(Err(crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::Cancelled)) => Some(Attempt::Cancelled),
                            Authorization::Stopped(Err(crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind::TimedOut)) => Some(Attempt::DeadlineExceeded),
                            _ => None,
                        };
                        match stop {
                            Some(stop) => {
                                return Err(SelfDenial::new(stop, "principal fact comparison"))
                            }
                            None => false,
                        }
                    }
                }
            }
            Self::Authorization { decision, .. } => decision
                .remains_equal_in(execution, runtime, snapshot)
                .map_err(|cause| {
                    SelfDenial::relational_read_stopped(cause, "authorization decision fact")
                })?,
        };
        Ok(equal)
    }
}

const DISAGREEMENT: &str = "decision facts disagree at one structural locator";

/// Equal structural observations may carry different read bounds. Retain the
/// one whose bound covers both; any other difference is a disagreement.
fn retained_observation(
    fact: SharedObservedFact,
    source: SharedObservedFact,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<SharedObservedFact, BindingFailure> {
    use crate::domain_computation::primary_graph::{
        application_attempt::WorthQueryApplicationObservedFact as Fact,
        source_facts::SourceFactsAllocation,
    };
    if let (
        Fact::SourceReadAuthorization { observation: first },
        Fact::SourceReadAuthorization {
            observation: second,
        },
    ) = (fact.as_ref(), source.as_ref())
    {
        let equal = first
            .same_truth(second, &mut |units| {
                SourceFactsAllocation::work(execution, units)
            })
            .map_err(|cause| {
                SelfDenial::relational_read_stopped(cause, "Source authorization role merge")
                    .into_decision_read_set_failure()
            })?;
        SourceFactsAllocation::work(execution, 0).map_err(|cause| {
            SelfDenial::relational_read_stopped(cause, "Source authorization role merge")
                .into_decision_read_set_failure()
        })?;
        return if equal {
            Ok(fact)
        } else {
            Err(binding_failure(DISAGREEMENT))
        };
    }
    if let (
        Fact::ManagedIndexSource { observation: first },
        Fact::ManagedIndexSource {
            observation: second,
        },
    ) = (fact.as_ref(), source.as_ref())
    {
        let equal = first
            .same_truth(
                second,
                &mut |units| SourceFactsAllocation::work(execution, units),
                &mut |bytes| execution.reserve_retained_memory(bytes).map_err(Into::into),
                &mut |cause| cause.into_native(),
                &mut |left, right| {
                    left.same_content(execution, right)
                        .map_err(|cause| cause.into_execution_cause())
                },
            )
            .map_err(|cause| {
                SelfDenial::relational_read_stopped(cause, "managed fact role merge")
                    .into_decision_read_set_failure()
            })?;
        SourceFactsAllocation::work(execution, 0).map_err(|cause| {
            SelfDenial::relational_read_stopped(cause, "managed fact role merge")
                .into_decision_read_set_failure()
        })?;
        return if equal {
            Ok(fact)
        } else {
            Err(binding_failure(DISAGREEMENT))
        };
    }
    if fact.as_ref() == source.as_ref() {
        return Ok(fact);
    }
    let mut merged = fact.as_ref().clone();
    if !merged.merge_equal_observation(source.as_ref()) {
        return Err(binding_failure(DISAGREEMENT));
    }
    Ok(if &merged == source.as_ref() {
        source
    } else {
        fact
    })
}

fn binding_failure(detail: &'static str) -> BindingFailure {
    BindingFailure::new(BindingKind::ProviderRejected, detail)
}

mod court_doors_for_test;
