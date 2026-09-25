//! Commit-snapshot readmission of an approval from its durable, descriptive basis.

use worth_query_installation::facade::ApplicationSchema;

use super::{
    foreign_runtime, stale_authorization, stale_principal, RevalidationObservationAxes,
    WorthQueryAuthorizationRevalidationObservation, WorthQueryCapabilityObservationSource,
};
use crate::domain_computation::authorization::{
    WorthQueryOperationAuthorizationDenial, WorthQueryWorkflowApprovalAuthorityBasis,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(in crate::domain_computation) fn readmit_workflow_approval_authority<
        Operation,
        Input,
        Scope,
    >(
        &self,
        authority: &WorthQueryWorkflowApprovalAuthorityBasis,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let product = admission
            .graph_work()
            .mutation_product()
            .ok_or_else(stale_authorization)?;
        self.readmit_workflow_approval_authority_on_product(
            authority,
            product,
            admission.graph_work_session_identity(),
        )
    }

    pub(in crate::domain_computation) fn readmit_workflow_approval_authority_on_product(
        &self,
        authority: &WorthQueryWorkflowApprovalAuthorityBasis,
        product: &crate::basis::WorthQueryProductBranchLease,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        if authority.principal().principal() != authority.request().principal() {
            return Err(stale_principal());
        }
        let installed = self.installed_capability_plan(authority.request())?;
        if authority.capability_authority_identity()
            != installed.capability_authority_identity().as_ref()
        {
            return Err(stale_authorization());
        }
        let sample = self.sample_capability_time(installed)?;
        let worth_foundational::facade::AspectValue::UInt64(now) = sample.value() else {
            return Err(stale_authorization());
        };
        if sample.timeline() != authority.timeline() {
            return Err(stale_authorization());
        }
        if *now > authority.expiry() {
            return Err(WorthQueryOperationAuthorizationDenial::new(
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::CapabilityExpired,
                "workflow approval",
            ));
        }
        let security = self
            .admit_product_security_basis(product)
            .map_err(|denial| {
                super::super::denial::product_security_basis_denial(
                    denial,
                    "workflow approval authorization",
                )
            })?;
        let graph = self.runtime.primary_graph().ok_or_else(foreign_runtime)?;
        graph.integration_handle().with_runtime_mut(|runtime| {
            let snapshot = security.snapshot_handle();
            if !authority
                .principal()
                .remains_current_in(runtime, snapshot, graph.layout())
            {
                return Err(stale_principal());
            }
            let observed = WorthQueryAuthorizationRevalidationObservation::from_axes(
                RevalidationObservationAxes {
                    session,
                    relational: runtime,
                    snapshot,
                    bridge: self.authorization.bridge(),
                    installed,
                    request: authority.request(),
                    sample: &sample,
                },
            )
            .observe_active_capability(Some(authority.grant()), None)?;
            (observed.decision().durable_lineage() == *authority.lineage())
                .then_some(())
                .ok_or_else(stale_authorization)?;
            if !authority.dependencies().primary_matches(observed.decision()) {
                return Err(stale_authorization());
            }
            if let Some(support) = authority.support() {
                if support.request().resource() != authority.request().resource()
                    || support.request().principal() != authority.request().principal()
                    || !support_matches_primary(authority, support)
                {
                    return Err(stale_authorization());
                }
                let installed = self.installed_capability_plan(support.request())?;
                if support.capability_authority_identity()
                    != installed.capability_authority_identity().as_ref()
                {
                    return Err(stale_authorization());
                }
                let sample = self.sample_capability_time(installed)?;
                let worth_foundational::facade::AspectValue::UInt64(support_now) = sample.value() else {
                    return Err(stale_authorization());
                };
                if sample.timeline() != support.timeline() {
                    return Err(stale_authorization());
                }
                if *support_now > support.expiry() {
                    return Err(WorthQueryOperationAuthorizationDenial::new(
                        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::CapabilityExpired,
                        "workflow approval support",
                    ));
                }
                let observed = WorthQueryAuthorizationRevalidationObservation::from_axes(
                    RevalidationObservationAxes {
                        session,
                        relational: runtime,
                        snapshot,
                        bridge: self.authorization.bridge(),
                        installed,
                        request: support.request(),
                        sample: &sample,
                    },
                )
                .observe_retained_capability(
                    support.posture(),
                    support.grant(),
                    None,
                )?;
                if observed.decision().durable_lineage() != *support.lineage() {
                    return Err(stale_authorization());
                }
                if !authority.dependencies().support_matches(observed.decision()) {
                    return Err(stale_authorization());
                }
            }
            Ok(())
        })
    }
}

fn support_matches_primary(
    authority: &WorthQueryWorkflowApprovalAuthorityBasis,
    support: &crate::domain_computation::authorization::workflow_approval_authority::WorthQueryWorkflowApprovalSupportBasis,
) -> bool {
    use crate::domain_computation::authorization::delegation_admission::WorthQueryCapabilityObservationPosture;
    use crate::domain_computation::authorization::WorthQueryCapabilitySupportRole;
    match support.role() {
        WorthQueryCapabilitySupportRole::DelegationTarget => {
            support.posture() == WorthQueryCapabilityObservationPosture::Active
        }
        WorthQueryCapabilitySupportRole::ElevationUpperBound => {
            support.posture() == WorthQueryCapabilityObservationPosture::UpperBound
                && support.request().capability_identity()
                    == authority.request().capability_identity()
                && support.capability_authority_identity()
                    == authority.capability_authority_identity()
                && support.grant() == authority.grant()
        }
    }
}
