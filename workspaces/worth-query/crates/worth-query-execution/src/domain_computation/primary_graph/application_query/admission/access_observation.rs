use super::super::{
    WorthQueryApplicationAuthorizationWorkEvidence, WorthQueryApplicationQueryAccessContext,
    WorthQueryApplicationQueryAdmissionDenial, WorthQueryApplicationQueryAdmissionDenialKind,
};
use super::denial::denial;
use crate::domain_computation::authorization::{
    WorthQueryPrincipalCurrentnessDependency, WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::primary_graph::{
    validate_freshness_at_snapshot, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_installation::facade::{
    TypedApplicationValue, WorthQueryInstalledApplicationQuery,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph::application_query) fn observe_application_query_access<
        Principal,
        PrincipalIdentity,
        Scope,
        Query,
        Parameters,
        QueryResult,
    >(
        &self,
        graph_work: &mut crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        access: &WorthQueryApplicationQueryAccessContext<
            '_,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        security: Option<&crate::domain_computation::primary_graph::product_operation::WorthQueryProductSecurityBasis>,
    ) -> Result<
        (
            WorthQueryRetainedAuthorizationDecisionFacts,
            WorthQueryApplicationAuthorizationWorkEvidence,
        ),
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        let session_identity = graph_work.identity();
        let scope = access.scope();
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                query.name(),
            )
        })?;
        let principal = access.principal();
        let principal_layout = graph
            .layout
            .principal_binding(principal.binding())
            .cloned()
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal,
                    principal.binding(),
                )
            })?;
        let expected_external_identity = principal
            .external_identity()
            .clone()
            .into_foundational_value();
        let principal_currentness = WorthQueryPrincipalCurrentnessDependency::capture(
            session_identity,
            principal,
            &principal_layout,
        );
        let entity_resolution = graph.retain_entity_resolution_context();
        let policy = graph.integration_handle().with_runtime_mut(|runtime| {
            let snapshot = if let Some(security) = security {
                std::borrow::Cow::Borrowed(security.snapshot_handle())
            } else {
                std::borrow::Cow::Owned(crate::domain_computation::primary_graph::exact_basis_access::open_current_main_snapshot(runtime)
                .map_err(|basis_denial| {
                    let kind = match basis_denial {
                        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::ActiveSnapshotCapacityExhausted {
                            maximum_active_snapshots,
                        } => WorthQueryApplicationQueryAdmissionDenialKind::ActiveSnapshotCapacityExhausted {
                            maximum_active_snapshots,
                        },
                        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted => {
                            WorthQueryApplicationQueryAdmissionDenialKind::RetentionCapacityExhausted
                        }
                        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted => {
                            WorthQueryApplicationQueryAdmissionDenialKind::RetentionIdentityExhausted
                        }
                        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted => {
                            WorthQueryApplicationQueryAdmissionDenialKind::SnapshotIdentityExhausted
                        }
                        _ => WorthQueryApplicationQueryAdmissionDenialKind::TruthViewUnavailable,
                    };
                    denial(kind, query.name())
                })?)
            };
            let result = if !graph_work.admits_snapshot(&snapshot) {
                Err(denial(
                    WorthQueryApplicationQueryAdmissionDenialKind::GraphWorkAdmissionUnavailable,
                    query.name(),
                ))
            } else {
                validate_freshness_at_snapshot(
                    runtime,
                    &snapshot,
                    principal,
                    &principal_layout,
                    &expected_external_identity,
                )
                .map_err(|_| {
                    denial(
                        WorthQueryApplicationQueryAdmissionDenialKind::StalePrincipal,
                        principal.binding(),
                    )
                })
                .and_then(|()| {
                    entity_resolution
                        .at_snapshot(
                            runtime,
                            &snapshot,
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .and_then(|truth| truth.validate_entity_freshness(scope))
                        .map_err(|_| {
                            denial(
                                WorthQueryApplicationQueryAdmissionDenialKind::StaleScope,
                                query.name(),
                            )
                        })
                })
                .and_then(|()| {
                    self.observe_query_authorization(
                        session_identity,
                        runtime,
                        snapshot.as_ref().clone(),
                        query,
                        access,
                    )
                    .map_err(map_authorization_denial)
                })
            };
            if let std::borrow::Cow::Owned(snapshot) = snapshot {
                crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            }
            result
        })?;
        let work = WorthQueryApplicationAuthorizationWorkEvidence::from_dependencies(&policy);
        let work = if security.is_some() {
            work.with_admission_security_product_resolution()
        } else {
            work
        };
        let authorization = if policy.is_empty() {
            WorthQueryRetainedAuthorizationDecisionFacts::principal(principal_currentness)
        } else {
            WorthQueryRetainedAuthorizationDecisionFacts::abilities(principal_currentness, policy)
        };
        Ok((authorization, work))
    }
}

fn map_authorization_denial(
    authorization: crate::domain_computation::primary_graph::WorthQueryOperationAuthorizationDenial,
) -> WorthQueryApplicationQueryAdmissionDenial {
    WorthQueryApplicationQueryAdmissionDenial::from_authorization(authorization)
}
