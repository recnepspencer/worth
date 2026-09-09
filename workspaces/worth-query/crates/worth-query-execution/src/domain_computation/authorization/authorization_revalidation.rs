mod currentness;
mod observation;
mod retained_authorization;
mod supporting_capability;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    decision_facts::WorthQueryObservedCommitBasis,
    delegation_admission::WorthQueryCapabilityObservationSource,
};
use super::{
    WorthQueryApplicationCommitAuthorization, WorthQueryCapabilityCommitBasis,
    WorthQueryCommitAuthorizationBasis, WorthQueryOperationAuthorizationDenial,
    WorthQueryOperationAuthorizationDenialKind, WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryApplicationCommitSerialization,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use currentness::*;
pub(in crate::domain_computation::authorization) use observation::WorthQueryAuthorizationRevalidationObservation;

struct RevalidationObservationAxes<'observation> {
    session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    relational: &'observation worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'observation worth_relational::facade::snapshots::SnapshotHandle,
    bridge: &'observation worth_runtime_bridge::facade::BridgeAuthorizationRuntime,
    installed: &'observation super::capability_registry::WorthQueryInstalledCapabilityPlan,
    request: &'observation super::WorthQueryRetainedCapabilityRequest,
    sample: &'observation super::WorthQueryRuntimeTimeSample,
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    fn readmit_admitted_non_capability_authorization<Operation, Input, Scope>(
        &self,
        admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let session = admission.graph_work_session_identity();
        let operation = admission.operation().to_owned();
        let product = admission
            .graph_work()
            .mutation_product()
            .map(crate::basis::WorthQueryProductBranchLease::retained_clone)
            .ok_or_else(|| WorthQueryOperationAuthorizationDenial::inconsistent(&operation))?;
        let security = self
            .admit_product_security_basis(&product)
            .map_err(|denial| super::denial::product_security_basis_denial(denial, &operation))?;
        let authorization = admission
            .authorization_mut()
            .ok_or_else(|| WorthQueryOperationAuthorizationDenial::inconsistent(&operation))?;
        if !authorization.belongs_to_session(session) {
            return Err(inconsistent_authorization());
        }
        if authorization.capability_authorization().is_some() {
            return Err(inconsistent_authorization());
        }
        let graph = self.runtime.primary_graph().ok_or_else(foreign_runtime)?;
        graph.integration_handle().with_runtime_mut(|runtime| {
            let snapshot = security.snapshot_handle();
            let current = validate_retained_currentness(
                authorization,
                runtime,
                snapshot,
                self.authorization.bridge(),
            );
            current
        })
    }

    pub(in crate::domain_computation) fn authorize_retained_idempotency<
        'serialization,
        'admission,
        Operation,
        Input,
        Scope,
    >(
        &self,
        admission: &'admission mut WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        serialization: &'serialization WorthQueryApplicationCommitSerialization<'_>,
    ) -> Result<
        WorthQueryApplicationCommitAuthorization<
            'serialization,
            'admission,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    > {
        if !self.refresh_admitted_capability_authorization(admission)? {
            self.readmit_admitted_non_capability_authorization(admission)?;
        }
        Ok(WorthQueryApplicationCommitAuthorization::mint(
            serialization,
            admission,
        ))
    }

    pub(in crate::domain_computation) fn authorize_idempotency_inspection<
        'serialization,
        'admission,
        Operation,
        Input,
        Scope,
    >(
        &self,
        admission: &'admission WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        serialization: &'serialization WorthQueryApplicationCommitSerialization<'_>,
    ) -> Result<
        WorthQueryApplicationCommitAuthorization<
            'serialization,
            'admission,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    > {
        self.validate_admitted_authorization(admission)?;
        Ok(WorthQueryApplicationCommitAuthorization::mint(
            serialization,
            admission,
        ))
    }

    pub(in crate::domain_computation::authorization) fn authorize_application_commit<
        'serialization,
        'admission,
        Operation,
        Input,
        Scope,
    >(
        &self,
        admission: &'admission WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
        basis: &WorthQueryCommitAuthorizationBasis,
        serialization: &'serialization WorthQueryApplicationCommitSerialization<'_>,
    ) -> Result<
        WorthQueryApplicationCommitAuthorization<
            'serialization,
            'admission,
            Schema,
            Operation,
            Input,
            Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    > {
        if self.runtime.authority_identity() != admission.runtime_authority() {
            return Err(foreign_runtime());
        }
        if basis.admission_identity() != admission.admission_identity()
            || !basis.belongs_to_session(admission.graph_work_session_identity())
        {
            return Err(WorthQueryOperationAuthorizationDenial::inconsistent(
                admission.operation(),
            ));
        }
        let product = admission.graph_work().mutation_product().ok_or_else(|| {
            WorthQueryOperationAuthorizationDenial::inconsistent(admission.operation())
        })?;
        self.readmit_commit_basis(basis, product)?;
        Ok(WorthQueryApplicationCommitAuthorization::mint(
            serialization,
            admission,
        ))
    }

    fn readmit_commit_basis(
        &self,
        basis: &WorthQueryCommitAuthorizationBasis,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        match basis {
            WorthQueryCommitAuthorizationBasis::Observed {
                authorization: observed,
                ..
            } => self.readmit_observed_commit_basis(observed, product),
            WorthQueryCommitAuthorizationBasis::Capability {
                authorization: capability,
                ..
            } => self.readmit_capability_commit_basis(capability, product),
        }
    }

    fn readmit_observed_commit_basis(
        &self,
        observed: &WorthQueryObservedCommitBasis,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let security = self
            .admit_product_security_basis(product)
            .map_err(|denial| {
                super::denial::product_security_basis_denial(
                    denial,
                    "observed commit authorization",
                )
            })?;
        let graph = self.runtime.primary_graph().ok_or_else(foreign_runtime)?;
        graph.integration_handle().with_runtime_mut(|runtime| {
            let snapshot = security.snapshot_handle();
            let current = validate_observed_currentness(
                observed,
                runtime,
                snapshot,
                self.authorization.bridge(),
            );
            current
        })
    }

    fn readmit_capability_commit_basis(
        &self,
        capability: &WorthQueryCapabilityCommitBasis,
        product: &crate::basis::WorthQueryProductBranchLease,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        let installed = self.installed_capability_plan(capability.request())?;
        if capability.capability_authority_identity()
            != installed.capability_authority_identity().as_ref()
        {
            return Err(stale_authorization());
        }
        let sample = self.sample_capability_time(installed)?;
        let security = self
            .admit_product_security_basis(product)
            .map_err(|denial| {
                super::denial::product_security_basis_denial(
                    denial,
                    "capability commit authorization",
                )
            })?;
        let graph = self.runtime.primary_graph().ok_or_else(foreign_runtime)?;
        graph.integration_handle().with_runtime_mut(|runtime| {
            let snapshot = security.snapshot_handle();
            let principal_current = capability.principal().remains_current_in(runtime, snapshot);
            let decision_current = capability.decision().remains_current_in(
                runtime,
                snapshot,
                self.authorization.bridge(),
            );
            let result = if !principal_current {
                Err(stale_principal())
            } else if !decision_current {
                Err(stale_authorization())
            } else {
                WorthQueryAuthorizationRevalidationObservation::from_axes(
                    RevalidationObservationAxes {
                        session: capability.decision().session_identity(),
                        relational: runtime,
                        snapshot,
                        bridge: self.authorization.bridge(),
                        installed,
                        request: capability.request(),
                        sample: &sample,
                    },
                )
                .observe_active_capability(Some(capability.grant()), Some(capability.decision()))
                .map(drop)
            };
            let result = result.and_then(|()| {
                self.readmit_capability_commit_support(capability.supporting(), runtime, snapshot)
            });
            result
        })
    }
}
