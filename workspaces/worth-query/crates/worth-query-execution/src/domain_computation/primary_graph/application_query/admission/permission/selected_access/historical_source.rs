//! C issues the principal and grants; the navigation's D proves source presence.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::ApplicationQueryNativeAccess,
    product_operation::document_history::HistoryProducerInvocation,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode,
};

impl<'a, S: ApplicationSchema, P, I, E> SelectedIssuedAccessRoot<'a, S, P, I, E> {
    pub(in crate::domain_computation::primary_graph::application_query) fn from_historical_source(
        current: &'a WorthQuerySelectedProductOperation<'_, S>,
        truth: &'a WorthQuerySelectedProductOperation<'_, S>,
        access: &WorthQueryApplicationQueryAccessContext<'a, S, P, I, E>,
        principal: &'a WorthQueryIssuedSelectedPrincipal<'_, '_, S, P, I>,
        history: HistoryProducerInvocation<'a>,
        request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        subject: &str,
    ) -> Result<Self, WorthQueryApplicationQueryAdmissionDenial> {
        if !std::ptr::eq(principal.selected(), current)
            || !std::ptr::eq(principal.principal(), access.principal())
            || !principal.request().same_request(request)
            || !history.admits_principal(&crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(principal.principal().principal_entity_id()))
            || !history.admits_request(request)
            || !history.admits_historical_truth(truth)
            || history.current().observation() != current.product().observation()
        {
            return Err(super::super::denial(
                WorthQueryApplicationQueryAdmissionDenialKind::StaleBasis,
                subject,
            ));
        }
        Ok(Self {
            principal: principal.principal(),
            scope: access.scope(),
            root: principal.issued_root(),
            snapshot: principal.issued_snapshot(),
            product: current.product().observation(),
            request: principal.request(),
            truth: Some(SelectedIssuedTruthRoot {
                product: truth.product().observation(),
                root: truth.product().relational_basis(),
                snapshot: truth.application_basis().snapshot_handle(),
            }),
            historical_source: Some(history),
        })
    }

    /// Called after the existing C authorization observer has succeeded. The
    /// historical posture never treats its descriptive scope as C-issued.
    pub(in crate::domain_computation::primary_graph::application_query::admission::permission) fn require_source_after_authorization(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<S>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &mut ApplicationQueryNativeAccess<'_>,
        subject: &str,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        let Some(history) = self.historical_source else {
            return Ok(());
        };
        let graph = runtime.runtime.primary_graph().ok_or_else(|| {
            super::super::denial(
                WorthQueryApplicationQueryAdmissionDenialKind::RuntimeSupportUnavailable,
                subject,
            )
        })?;
        native.with_runtime(
            graph,
            execution,
            || {
                super::super::denial(
                    WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
                    subject,
                )
            },
            |relational| {
                let truth = self
                    .truth
                    .as_ref()
                    .expect("historical admission binds its exact D");
                history.require_historical_source(
                    execution, relational, self.scope, truth.root, runtime, subject,
                )?;
                // D may have progressed while another family settled. Its exact
                // data basis must still contain this generation and identity.
                let truth = self
                    .truth
                    .as_ref()
                    .expect("historical admission binds its D");
                graph
                    .retain_entity_resolution_context()
                    .at_snapshot(
                        relational,
                        truth.snapshot,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .and_then(|reader| reader.validate_entity_freshness(execution, self.scope))
                    .map_err(|cause| {
                        super::super::denial(
                            super::super::super::access_observation::freshness_denial::entity(
                                cause.kind(),
                            ),
                            subject,
                        )
                    })
            },
        )
    }
}
