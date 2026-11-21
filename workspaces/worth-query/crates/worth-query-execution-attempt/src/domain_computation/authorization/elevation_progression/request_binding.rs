use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::ApplicationOperationProgramTarget;
use worth_relational::facade::identity::KindId;

use super::WorthQueryElevationUpperBound;
use crate::domain_computation::authorization::WorthQueryRetainedCapabilitySupport;

#[derive(Debug)]
pub struct WorthQueryElevationRequestBinding {
    pub runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    pub branch: worth_relational::facade::history::BranchId,
    pub capability_identity: [u8; 32],
    pub capability_authority_identity: Arc<str>,
    pub observed: WorthQueryObservedElevationSupport,
    pub elevation_kind: KindId,
    pub review_kind: KindId,
    pub elevation_key: String,
    pub elevation_identity_field: AspectFieldLocator,
    pub elevation_identity: AspectValue,
    pub reason_field: AspectFieldLocator,
    pub reason: AspectValue,
    pub status_field: AspectFieldLocator,
    pub requested_status: AspectValue,
    pub not_before_field: AspectFieldLocator,
    pub issued_at: AspectValue,
    pub not_after_field: AspectFieldLocator,
    pub expires_at: AspectValue,
    pub review_key: String,
    pub review_identity_field: AspectFieldLocator,
    pub review_identity: AspectValue,
    pub review_type_field: AspectFieldLocator,
    pub review_type: AspectValue,
    pub review_status_field: AspectFieldLocator,
    pub review_required_status: AspectValue,
    pub requester_relation: KindId,
    pub grant_relation: KindId,
    pub resource_relation: Option<KindId>,
    pub review_relation: KindId,
    pub review_scope_relation: KindId,
    pub required_program_targets:
        Vec<ApplicationOperationProgramTarget>,
    pub lifecycle_effect:
        Option<worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect>,
}


#[derive(Debug)]
pub struct WorthQueryObservedElevationSupport {
    upper_bound: WorthQueryElevationUpperBound,
    supporting: WorthQueryRetainedCapabilitySupport,
}

pub struct WorthQueryCurrentElevationSupport {
    sample: crate::domain_computation::authorization::WorthQueryRuntimeTimeSample,
    observed: crate::domain_computation::authorization::capability_observation::WorthQueryObservedCapabilityDecision,
}

impl WorthQueryCurrentElevationSupport {
    pub const fn new(
        sample: crate::domain_computation::authorization::WorthQueryRuntimeTimeSample,
        observed: crate::domain_computation::authorization::capability_observation::WorthQueryObservedCapabilityDecision,
    ) -> Self {
        Self { sample, observed }
    }

    pub(super) fn apply<Schema, Capability, Operation, Input>(
        self,
        requested: &mut WorthQueryElevationRequestBinding,
        access: &mut crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        subject: &str,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryRuntimeTimeSample,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Input:
            worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
                Schema,
                Capability,
            >,
    {
        let decision = self
            .observed
            .into_decision_for_grant(requested.supporting().grant())
            .map_err(|()| inconsistent_support(subject))?;
        let current_sample = self.sample.clone();
        requested
            .supporting_mut()
            .replace_current_session(access.graph_work_session_identity(), self.sample, decision)
            .map_err(|()| inconsistent_support(subject))?;
        access
            .retain_observed_support(requested.supporting().retained_for_operation())
            .map_err(|()| inconsistent_support(subject))?;
        Ok(current_sample)
    }
}

fn inconsistent_support(
    subject: &str,
) -> crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial {
    crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::InconsistentDecision),
        subject,
    )
}

impl WorthQueryObservedElevationSupport {
    pub const fn new(
        upper_bound: WorthQueryElevationUpperBound,
        supporting: WorthQueryRetainedCapabilitySupport,
    ) -> Self {
        Self {
            upper_bound,
            supporting,
        }
    }
    pub fn retained_for_operation(self) -> Self {
        Self {
            upper_bound: self.upper_bound,
            supporting: self.supporting.retained_for_operation(),
        }
    }
}

impl WorthQueryElevationRequestBinding {
    pub const fn runtime_authority(
        &self,
    ) -> &crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        &self.runtime_authority
    }
    pub const fn branch(
        &self,
    ) -> &worth_relational::facade::history::BranchId {
        &self.branch
    }
    pub const fn capability_identity(&self) -> [u8; 32] {
        self.capability_identity
    }
    pub fn capability_authority_identity(&self) -> &str {
        &self.capability_authority_identity
    }
    pub const fn elevation_kind(&self) -> KindId {
        self.elevation_kind
    }
    pub const fn review_kind(&self) -> KindId {
        self.review_kind
    }
    pub fn elevation_key(&self) -> &str {
        &self.elevation_key
    }
    pub const fn elevation_identity_field(
        &self,
    ) -> &AspectFieldLocator {
        &self.elevation_identity_field
    }
    pub const fn elevation_identity(&self) -> &AspectValue {
        &self.elevation_identity
    }
    pub const fn reason_field(&self) -> &AspectFieldLocator {
        &self.reason_field
    }
    pub const fn reason(&self) -> &AspectValue {
        &self.reason
    }
    pub const fn status_field(&self) -> &AspectFieldLocator {
        &self.status_field
    }
    pub const fn requested_status(&self) -> &AspectValue {
        &self.requested_status
    }
    pub const fn not_before_field(&self) -> &AspectFieldLocator {
        &self.not_before_field
    }
    pub const fn issued_at(&self) -> &AspectValue {
        &self.issued_at
    }
    pub const fn not_after_field(&self) -> &AspectFieldLocator {
        &self.not_after_field
    }
    pub const fn expires_at(&self) -> &AspectValue {
        &self.expires_at
    }
    pub fn review_key(&self) -> &str {
        &self.review_key
    }
    pub const fn review_identity_field(&self) -> &AspectFieldLocator {
        &self.review_identity_field
    }
    pub const fn review_identity(&self) -> &AspectValue {
        &self.review_identity
    }
    pub const fn review_type_field(&self) -> &AspectFieldLocator {
        &self.review_type_field
    }
    pub const fn review_type(&self) -> &AspectValue {
        &self.review_type
    }
    pub const fn review_status_field(&self) -> &AspectFieldLocator {
        &self.review_status_field
    }
    pub const fn review_required_status(&self) -> &AspectValue {
        &self.review_required_status
    }
    pub const fn requester_relation(&self) -> KindId {
        self.requester_relation
    }
    pub const fn grant_relation(&self) -> KindId {
        self.grant_relation
    }
    pub const fn resource_relation(&self) -> Option<KindId> {
        self.resource_relation
    }
    pub const fn review_relation(&self) -> KindId {
        self.review_relation
    }
    pub const fn review_scope_relation(&self) -> KindId {
        self.review_scope_relation
    }
    pub fn required_program_targets(
        &self,
    ) -> &[ApplicationOperationProgramTarget] {
        &self.required_program_targets
    }
    pub const fn lifecycle_effect(&self) -> Option<&worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect>{
        self.lifecycle_effect.as_ref()
    }
    pub const fn requester(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.observed.upper_bound.requester()
    }

    pub const fn resource(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.observed.upper_bound.resource()
    }

    pub const fn grant(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.observed.upper_bound.grant()
    }

    pub const fn upper_bound(
        &self,
    ) -> &WorthQueryElevationUpperBound {
        &self.observed.upper_bound
    }
    pub const fn supporting(
        &self,
    ) -> &WorthQueryRetainedCapabilitySupport {
        &self.observed.supporting
    }
    pub fn supporting_mut(&mut self) -> &mut WorthQueryRetainedCapabilitySupport {
        &mut self.observed.supporting
    }

    pub fn apply_current_support<
        Schema,
        Capability,
        Operation,
        Input,
    >(
        &mut self,
        current: WorthQueryCurrentElevationSupport,
        access: &mut crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        subject: &str,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryRuntimeTimeSample,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
        Input:
            worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
                Schema,
                Capability,
            >,
    {
        current.apply(self, access, subject)
    }
}
