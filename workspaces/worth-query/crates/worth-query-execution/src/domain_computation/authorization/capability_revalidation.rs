//! Current capability re-admission at privileged transitions.
mod installed_basis;

use worth_query_installation::facade::ApplicationSchema;

use super::delegation_admission::WorthQueryCapabilityObservationSource;
use super::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryRetainedCapabilityAuthorization,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

mod observation;
pub(in crate::domain_computation::authorization) use observation::WorthQueryCapabilityRevalidationObservation;

struct WorthQueryCapabilityRefresh<'refresh> {
    session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    product: &'refresh crate::basis::WorthQueryProductObservationLease,
    installed: &'refresh super::capability_registry::WorthQueryInstalledCapabilityPlan,
    sample: super::WorthQueryRuntimeTimeSample,
}

enum WorthQueryCapabilityRefreshTime<'sample> {
    Fresh,
    ReuseWhenTimelineMatches(&'sample super::WorthQueryRuntimeTimeSample),
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation) fn refresh_capability_authorization_for_graph_work(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authorization: &mut WorthQueryRetainedCapabilityAuthorization,
        graph_work: &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let product = graph_work.product().ok_or_else(|| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                "retained capability authorization",
            )
        })?;
        self.refresh_capability_authorization(
            execution,
            authorization,
            graph_work.identity(),
            product,
        )
    }

    pub(super) fn refresh_admitted_capability_authorization<Operation, Input, Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &mut super::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    ) -> Result<bool, WorthQueryOperationAuthorizationDenial> {
        let has_capability = admission
            .authorization()
            .ok_or_else(|| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                    admission.operation(),
                )
            })?
            .capability_authorization()
            .is_some();
        if !has_capability {
            return Ok(false);
        }
        let session_identity = admission.graph_work_session_identity();
        let operation = admission.operation().to_owned();
        let product = admission
            .graph_work()
            .mutation_product()
            .map(crate::basis::WorthQueryProductBranchLease::retained_clone)
            .ok_or_else(|| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                    &operation,
                )
            })?;
        let authorization = admission.authorization_mut().ok_or_else(|| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                &operation,
            )
        })?;
        if !authorization.belongs_to_session(session_identity) {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                "retained capability authorization",
            ));
        }
        let capability = authorization
            .capability_authorization_mut()
            .ok_or_else(|| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                    "retained capability authorization",
                )
            })?;
        self.refresh_capability_authorization(
            execution,
            capability,
            session_identity,
            product.read_lease_ref(),
        )?;
        Ok(true)
    }

    fn refresh_capability_authorization(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authorization: &mut WorthQueryRetainedCapabilityAuthorization,
        session_identity: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        product: &crate::basis::WorthQueryProductObservationLease,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let refresh = self.prepare_capability_refresh(
            authorization.request(),
            authorization.capability_authority_identity(),
            session_identity,
            product,
            WorthQueryCapabilityRefreshTime::Fresh,
        )?;
        let observed =
            self.observe_active_capability_refresh(execution, authorization, &refresh)?;
        self.replace_active_capability_decision(authorization, &refresh, observed)?;
        self.refresh_supporting_authorization(execution, authorization, &refresh)
    }

    fn prepare_capability_refresh<'refresh>(
        &'refresh self,
        request: &WorthQueryRetainedCapabilityRequest,
        authority_identity: &str,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
        product: &'refresh crate::basis::WorthQueryProductObservationLease,
        time: WorthQueryCapabilityRefreshTime<'_>,
    ) -> Result<WorthQueryCapabilityRefresh<'refresh>, WorthQueryOperationAuthorizationDenial> {
        let installed = self.installed_capability_plan(request)?;
        if authority_identity != installed.capability_authority_identity().as_ref() {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::PolicyNotInstalled,
                installed.contract().name(),
            ));
        }
        let sample = match time {
            WorthQueryCapabilityRefreshTime::ReuseWhenTimelineMatches(sample)
                if installed.request().timeline == sample.timeline() =>
            {
                sample.clone()
            }
            WorthQueryCapabilityRefreshTime::Fresh
            | WorthQueryCapabilityRefreshTime::ReuseWhenTimelineMatches(_) => {
                self.sample_capability_time(installed)?
            }
        };
        Ok(WorthQueryCapabilityRefresh {
            session,
            product,
            installed,
            sample,
        })
    }

    fn observe_active_capability_refresh(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authorization: &WorthQueryRetainedCapabilityAuthorization,
        refresh: &WorthQueryCapabilityRefresh<'_>,
    ) -> Result<
        super::capability_observation::WorthQueryObservedCapabilityDecision,
        WorthQueryOperationAuthorizationDenial,
    > {
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
                refresh.installed.contract().name(),
            )
        })?;
        let security = self
            .admit_product_security_basis(execution, refresh.product)
            .map_err(|denial| {
                super::denial::product_security_basis_denial(
                    denial,
                    refresh.installed.contract().name(),
                )
            })?;
        let observed = graph
            .integration_handle()
            .with_runtime_mut(execution, |runtime| {
                let snapshot = security.snapshot_handle();
                let result = self
                    .validate_active_capability_currentness(
                        execution,
                        runtime,
                        snapshot,
                        authorization,
                        refresh,
                    )
                    .and_then(|()| {
                        WorthQueryCapabilityRevalidationObservation::new(
                            refresh.session,
                            runtime,
                            snapshot,
                            self.authorization.bridge(),
                            refresh.installed,
                            authorization.request(),
                            &refresh.sample,
                        )
                        .observe_active_capability(
                            execution,
                            Some(authorization.grant()),
                            Some(authorization.decision()),
                        )
                    });
                result
            })?;
        Ok(observed)
    }

    fn validate_active_capability_currentness(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        authorization: &WorthQueryRetainedCapabilityAuthorization,
        refresh: &WorthQueryCapabilityRefresh<'_>,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if snapshot.branch_id() != refresh.product.relational_basis().identity().branch_id() {
            return Err(inconsistent_refresh(refresh));
        }
        if !authorization
            .principal()
            .remains_current_in(execution, runtime, snapshot)
            .map_err(|kind| {
                WorthQueryOperationAuthorizationDenial::new(kind, "principal currentness")
            })?
        {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::StalePrincipal,
                refresh.installed.contract().name(),
            ));
        }
        if !authorization
            .decision()
            .remains_current_in(execution, runtime, snapshot, self.authorization.bridge())
            .map_err(|cause| {
                crate::domain_computation::authorization::denial::relational_read_denial(
                    cause,
                    "authorization currentness",
                )
            })?
        {
            return Err(denial(
                WorthQueryOperationAuthorizationDenialKind::StaleAuthorization,
                refresh.installed.contract().name(),
            ));
        }
        Ok(())
    }

    fn replace_active_capability_decision(
        &self,
        authorization: &mut WorthQueryRetainedCapabilityAuthorization,
        refresh: &WorthQueryCapabilityRefresh<'_>,
        observed: super::capability_observation::WorthQueryObservedCapabilityDecision,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let grant = authorization.grant();
        let (fact, expiry) = observed.into_refresh_for_grant(grant).map_err(|()| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                refresh.installed.contract().name(),
            )
        })?;
        authorization
            .replace_current_session_decision(
                refresh.session,
                refresh.installed.capability_authority_identity().as_ref(),
                grant,
                refresh.sample.clone(),
                expiry,
                fact,
            )
            .map_err(|()| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                    refresh.installed.contract().name(),
                )
            })
    }

    fn refresh_supporting_authorization(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        authorization: &mut WorthQueryRetainedCapabilityAuthorization,
        primary: &WorthQueryCapabilityRefresh<'_>,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let Some(supporting) = authorization.supporting_mut() else {
            return Ok(());
        };
        let refresh = self.prepare_capability_refresh(
            supporting.request(),
            supporting.capability_authority_identity(),
            primary.session,
            primary.product,
            WorthQueryCapabilityRefreshTime::ReuseWhenTimelineMatches(&primary.sample),
        )?;
        let observed =
            self.observe_supporting_capability_refresh(execution, supporting, &refresh)?;
        let decision = observed
            .into_decision_for_grant(supporting.grant())
            .map_err(|()| inconsistent_refresh(&refresh))?;
        let replacement_denial = inconsistent_refresh(&refresh);
        supporting
            .replace_current_session(refresh.session, refresh.sample, decision)
            .map_err(|()| replacement_denial)
    }

    fn observe_supporting_capability_refresh(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        supporting: &super::WorthQueryRetainedCapabilitySupport,
        refresh: &WorthQueryCapabilityRefresh<'_>,
    ) -> Result<
        super::capability_observation::WorthQueryObservedCapabilityDecision,
        WorthQueryOperationAuthorizationDenial,
    > {
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryOperationAuthorizationDenialKind::ForeignRuntime,
                refresh.installed.contract().name(),
            )
        })?;
        let security = self
            .admit_product_security_basis(execution, refresh.product)
            .map_err(|denial| {
                super::denial::product_security_basis_denial(
                    denial,
                    refresh.installed.contract().name(),
                )
            })?;
        graph
            .integration_handle()
            .with_runtime_mut(execution, |runtime| {
                let snapshot = security.snapshot_handle();
                let result = WorthQueryCapabilityRevalidationObservation::new(
                    refresh.session,
                    runtime,
                    snapshot,
                    self.authorization.bridge(),
                    refresh.installed,
                    supporting.request(),
                    &refresh.sample,
                )
                .observe_retained_capability(
                    execution,
                    supporting.posture(),
                    supporting.grant(),
                    Some(supporting.decision()),
                );
                result
            })
    }
}

fn inconsistent_refresh(
    refresh: &WorthQueryCapabilityRefresh<'_>,
) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
        refresh.installed.contract().name(),
    )
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
