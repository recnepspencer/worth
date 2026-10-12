//! Revalidation of the supporting capability carried by a commit basis.

use worth_query_installation::facade::ApplicationSchema;

use super::{
    stale_authorization, WorthQueryAuthorizationRevalidationObservation,
    WorthQueryCapabilityObservationSource,
};
use crate::domain_computation::authorization::{
    WorthQueryCapabilitySupportCommitBasis, WorthQueryOperationAuthorizationDenial,
    WorthQueryRetainedCapabilityRequest, WorthQueryRetainedCapabilitySupport,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

trait CapabilitySupportRevalidation {
    fn decision(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryAuthorizationDecisionFact;
    fn capability_authority_identity(&self) -> &str;
    fn grant(&self) -> worth_relational::facade::identity::EntityId;
    fn request(&self) -> &WorthQueryRetainedCapabilityRequest;
    fn posture(&self) -> crate::domain_computation::authorization::delegation_admission::WorthQueryCapabilityObservationPosture;
}

macro_rules! impl_support_revalidation {
    ($support:ty) => {
        impl CapabilitySupportRevalidation for $support {
            fn decision(&self) -> &crate::domain_computation::authorization::WorthQueryAuthorizationDecisionFact {
                self.decision()
            }

            fn capability_authority_identity(&self) -> &str {
                self.capability_authority_identity()
            }

            fn grant(&self) -> worth_relational::facade::identity::EntityId {
                self.grant()
            }

            fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
                self.request()
            }

            fn posture(&self) -> crate::domain_computation::authorization::delegation_admission::WorthQueryCapabilityObservationPosture {
                self.posture()
            }
        }
    };
}

impl_support_revalidation!(WorthQueryCapabilitySupportCommitBasis);
impl_support_revalidation!(WorthQueryRetainedCapabilitySupport);

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub(super) fn readmit_capability_commit_support(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        supporting: Option<&WorthQueryCapabilitySupportCommitBasis>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        self.readmit_capability_support(execution, supporting, runtime, snapshot)
    }

    pub(super) fn readmit_retained_capability_support(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        supporting: Option<&WorthQueryRetainedCapabilitySupport>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        self.readmit_capability_support(execution, supporting, runtime, snapshot)
    }

    fn readmit_capability_support<Support>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        supporting: Option<&Support>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial>
    where
        Support: CapabilitySupportRevalidation,
    {
        let Some(supporting) = supporting else {
            return Ok(());
        };
        let installed = self.installed_capability_plan(supporting.request())?;
        if supporting.capability_authority_identity()
            != installed.capability_authority_identity().as_ref()
            || !supporting.decision().remains_current_in(
                execution,
                runtime,
                snapshot,
                self.authorization.bridge(),
            ).map_err(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::relational_read_stop)?
        {
            return Err(stale_authorization());
        }
        let sample = self.sample_capability_time(installed)?;
        WorthQueryAuthorizationRevalidationObservation::from_axes(
            super::RevalidationObservationAxes {
                session: supporting.decision().session_identity(),
                relational: runtime,
                snapshot,
                bridge: self.authorization.bridge(),
                installed,
                request: supporting.request(),
                sample: &sample,
            },
        )
        .observe_retained_capability(
            execution,
            supporting.posture(),
            supporting.grant(),
            Some(supporting.decision()),
        )
        .map(drop)
    }
}
