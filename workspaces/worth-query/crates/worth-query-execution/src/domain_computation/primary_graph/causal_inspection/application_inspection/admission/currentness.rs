//! Current original-branch authority is checked before lookup and before release.
use super::*;
use crate::domain_computation::primary_graph::{
    admitted_capture::{checkpoint, run_retained_stage, RetainedStorageDenial},
    WorthQueryPrincipalResolutionMode,
};
use worth_execution::MapKernelFailure;
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema, Scope> InspectionRead<'_, '_, Schema, Scope> {
    pub(super) fn check(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), InspectionAdmissionDenial> {
        require_request(execution, self.request)?;
        for governance in std::iter::once(&mut self.governance).chain(self.diagnostic.iter_mut()) {
            if let Some(authorization) = governance.authorization_mut() {
                self.runtime
                    .refresh_capability_authorization_for_graph_work(
                        execution,
                        authorization,
                        &self.session,
                    )
                    .map_err(InspectionAdmissionDenial::Capability)?;
            }
            if !governance.authorization_belongs_to_session(self.session.identity())
                || !governance.inspection_matches(
                    &self.session,
                    &self.target,
                    &self.target.parameters(),
                    self.principal,
                    self.scope.entity_id(),
                )
            {
                return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
            }
        }
        if !self
            .authorization
            .belongs_to_session(self.session.identity())
            || self.target.before().is_some() != self.diagnostic.is_some()
        {
            return Err(InspectionAdmissionDenial::ForeignIssuedAccess);
        }
        let product = self
            .session
            .authorization_product()
            .ok_or(InspectionAdmissionDenial::ForeignIssuedAccess)?;
        let security = self
            .runtime
            .admit_product_security_basis(execution, product)
            .map_err(InspectionAdmissionDenial::Security)?;
        let graph = self
            .runtime
            .runtime
            .primary_graph()
            .ok_or(InspectionAdmissionDenial::MissingGraph)?;
        let resolution = graph.retain_entity_resolution_context();
        let read = run_retained_stage(execution, self.memory, |context, memory| {
            checkpoint(context, 1).map_err(MapKernelFailure::Stop)?;
            let bytes = self
                .basis
                .retained_clone_heap_bytes()
                .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
            checkpoint(context, bytes).map_err(MapKernelFailure::Stop)?;
            if let Some(memory) = memory {
                memory.resize(bytes).map_err(MapKernelFailure::Domain)?;
            }
            Ok(self.session.execute_inspection_read(
                execution,
                &self.target,
                &self.basis,
                |relational, _| {
                    let snapshot = security.snapshot_handle();
                    self.authorization
                        .validate_currentness_in(
                            execution,
                            relational,
                            snapshot,
                            self.runtime.authorization.bridge(),
                        )
                        .map_err(InspectionAdmissionDenial::CurrentAuthorization)?;
                    for governance in
                        std::iter::once(&self.governance).chain(self.diagnostic.iter())
                    {
                        if let Some(authorization) = governance.authorization() {
                            authorization
                                .validate_currentness_in(
                                    execution,
                                    relational,
                                    snapshot,
                                    self.runtime.authorization.bridge(),
                                )
                                .map_err(InspectionAdmissionDenial::CurrentAuthorization)?;
                        }
                    }
                    resolution
                        .at_snapshot(
                            relational,
                            snapshot,
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .and_then(|truth| truth.validate_entity_freshness(execution, self.scope))
                        .map_err(InspectionAdmissionDenial::Scope)
                },
            ))
        })
        .map_err(InspectionAdmissionDenial::Storage)?;
        let (checked, proof) = read.map_err(InspectionAdmissionDenial::Read)?;
        checked?;
        self.proof = Some(proof);
        require_request(execution, self.request)
    }
}
