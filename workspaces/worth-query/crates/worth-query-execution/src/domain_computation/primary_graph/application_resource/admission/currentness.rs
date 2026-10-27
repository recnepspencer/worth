use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryOperationAuthorizationDenial, WorthQueryPrincipalResolutionMode,
};
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema, Scope> ResourceActionRead<'_, Schema, Scope> {
    /// Recheck before protected row/provider access and again before terminal
    /// handoff. The actual security owner selects current original-branch truth.
    pub(in crate::domain_computation::primary_graph::application_resource) fn check(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), ResourceAdmissionDenial> {
        require_request(execution, self.request)?;
        if let Some(authorization) = self.governance.authorization_mut() {
            self.runtime
                .refresh_capability_authorization_for_graph_work(
                    execution,
                    authorization,
                    &self.session,
                )
                .map_err(ResourceAdmissionDenial::DisclosureAuthorization)?;
        }
        if !self
            .authorization
            .belongs_to_session(self.session.identity())
            || !self
                .governance
                .authorization_belongs_to_session(self.session.identity())
            || !self.governance.resource_matches(
                &self.session,
                &self.contract,
                self.contract.purpose(),
                &self.parameters,
                self.principal,
                self.scope.entity_id(),
            )
        {
            return Err(ResourceAdmissionDenial::ForeignIssuedAccess);
        }
        let product = self
            .session
            .authorization_product()
            .ok_or(ResourceAdmissionDenial::ForeignIssuedAccess)?;
        let security = self
            .runtime
            .admit_product_security_basis(execution, product)
            .map_err(ResourceAdmissionDenial::Security)?;
        let graph = self
            .runtime
            .runtime
            .primary_graph()
            .ok_or(ResourceAdmissionDenial::MissingGraph)?;
        let resolution = graph.retain_entity_resolution_context();
        let (checked, proof) = self
            .session
            .execute_resource_read(execution, &self.basis, |relational, _| {
                let snapshot = security.snapshot_handle();
                self.authorization
                    .validate_currentness_in(
                        execution,
                        relational,
                        snapshot,
                        self.runtime.authorization.bridge(),
                    )
                    .map_err(|kind| {
                        ResourceAdmissionDenial::Authorization(
                            WorthQueryOperationAuthorizationDenial::new(kind, "resource action"),
                        )
                    })?;
                if let Some(authorization) = self.governance.authorization() {
                    authorization
                        .validate_currentness_in(
                            execution,
                            relational,
                            snapshot,
                            self.runtime.authorization.bridge(),
                        )
                        .map_err(|kind| {
                            ResourceAdmissionDenial::DisclosureAuthorization(
                                WorthQueryOperationAuthorizationDenial::new(
                                    kind,
                                    "resource disclosure",
                                ),
                            )
                        })?;
                }
                resolution
                    .at_snapshot(
                        relational,
                        snapshot,
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .and_then(|truth| truth.validate_entity_freshness(execution, self.scope))
                    .map_err(ResourceAdmissionDenial::Scope)
            })
            .map_err(ResourceAdmissionDenial::Read)?;
        checked?;
        if self
            .contract
            .disclosure_slots()
            .iter()
            .any(|slot| !self.governance.is_disclosed(slot))
        {
            return Err(ResourceAdmissionDenial::FacetDenied);
        }
        self.proof = Some(proof);
        require_request(execution, self.request)
    }

    pub(in crate::domain_computation::primary_graph::application_resource) fn terminal(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<ResourceActionTerminal, ResourceAdmissionDenial> {
        self.check(execution)?;
        Ok(ResourceActionTerminal {
            session: self.session,
            proof: self
                .proof
                .expect("successful Resource currentness carries the actual read proof"),
            _authorization: self.authorization,
            _governance: self.governance,
            _admission: self._admission,
        })
    }
}
