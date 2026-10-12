use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use std::sync::Arc;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::ApplicationOperationProgramTarget;
use worth_relational::facade::identity::KindId;

use super::WorthQueryElevationUpperBound;
use crate::domain_computation::authorization::WorthQueryRetainedCapabilitySupport;

#[derive(Debug)]
pub(in crate::domain_computation) struct WorthQueryElevationRequestBinding {
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    branch: worth_relational::facade::history::BranchId,
    capability_identity: [u8; 32],
    capability_authority_identity: Arc<str>,
    observed: WorthQueryObservedElevationSupport,
    elevation_kind: KindId,
    review_kind: KindId,
    elevation_key: String,
    elevation_identity_field: AspectFieldLocator,
    elevation_identity: AspectValue,
    reason_field: AspectFieldLocator,
    reason: AspectValue,
    status_field: AspectFieldLocator,
    requested_status: AspectValue,
    not_before_field: AspectFieldLocator,
    issued_at: AspectValue,
    not_after_field: AspectFieldLocator,
    expires_at: AspectValue,
    review_key: String,
    review_identity_field: AspectFieldLocator,
    review_identity: AspectValue,
    review_type_field: AspectFieldLocator,
    review_type: AspectValue,
    review_status_field: AspectFieldLocator,
    review_required_status: AspectValue,
    requester_relation: KindId,
    grant_relation: KindId,
    resource_relation: Option<KindId>,
    review_relation: KindId,
    review_scope_relation: KindId,
    required_program_targets:
        Vec<ApplicationOperationProgramTarget>,
    lifecycle_effect:
        Option<worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect>,
}

mod construction;
pub(super) use construction::bind_request;

#[derive(Debug)]
pub(in crate::domain_computation) struct WorthQueryObservedElevationSupport {
    upper_bound: WorthQueryElevationUpperBound,
    supporting: WorthQueryRetainedCapabilitySupport,
}

pub(in crate::domain_computation) struct WorthQueryCurrentElevationSupport {
    sample: crate::domain_computation::authorization::WorthQueryRuntimeTimeSample,
    observed: crate::domain_computation::authorization::capability_observation::WorthQueryObservedCapabilityDecision,
}

impl WorthQueryCurrentElevationSupport {
    pub(in crate::domain_computation::authorization) const fn new(
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
    pub(in crate::domain_computation::authorization) const fn new(
        upper_bound: WorthQueryElevationUpperBound,
        supporting: WorthQueryRetainedCapabilitySupport,
    ) -> Self {
        Self {
            upper_bound,
            supporting,
        }
    }
    pub(in crate::domain_computation) fn retained_for_operation(self) -> Self {
        Self {
            upper_bound: self.upper_bound,
            supporting: self.supporting.retained_for_operation(),
        }
    }
}

impl WorthQueryElevationRequestBinding {
    pub(in crate::domain_computation) const fn runtime_authority(
        &self,
    ) -> &crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity {
        &self.runtime_authority
    }
    pub(in crate::domain_computation) const fn branch(
        &self,
    ) -> &worth_relational::facade::history::BranchId {
        &self.branch
    }
    pub(in crate::domain_computation) const fn capability_identity(&self) -> [u8; 32] {
        self.capability_identity
    }
    pub(in crate::domain_computation) fn capability_authority_identity(&self) -> &str {
        &self.capability_authority_identity
    }
    pub(in crate::domain_computation) const fn elevation_kind(&self) -> KindId {
        self.elevation_kind
    }
    pub(in crate::domain_computation) const fn review_kind(&self) -> KindId {
        self.review_kind
    }
    pub(in crate::domain_computation) fn elevation_key(&self) -> &str {
        &self.elevation_key
    }
    pub(in crate::domain_computation) const fn elevation_identity_field(
        &self,
    ) -> &AspectFieldLocator {
        &self.elevation_identity_field
    }
    pub(in crate::domain_computation) const fn elevation_identity(&self) -> &AspectValue {
        &self.elevation_identity
    }
    pub(in crate::domain_computation) const fn reason_field(&self) -> &AspectFieldLocator {
        &self.reason_field
    }
    pub(in crate::domain_computation) const fn reason(&self) -> &AspectValue {
        &self.reason
    }
    pub(in crate::domain_computation) const fn status_field(&self) -> &AspectFieldLocator {
        &self.status_field
    }
    pub(in crate::domain_computation) const fn requested_status(&self) -> &AspectValue {
        &self.requested_status
    }
    pub(in crate::domain_computation) const fn not_before_field(&self) -> &AspectFieldLocator {
        &self.not_before_field
    }
    pub(in crate::domain_computation) const fn issued_at(&self) -> &AspectValue {
        &self.issued_at
    }
    pub(in crate::domain_computation) const fn not_after_field(&self) -> &AspectFieldLocator {
        &self.not_after_field
    }
    pub(in crate::domain_computation) const fn expires_at(&self) -> &AspectValue {
        &self.expires_at
    }
    pub(in crate::domain_computation) fn review_key(&self) -> &str {
        &self.review_key
    }
    pub(in crate::domain_computation) const fn review_identity_field(&self) -> &AspectFieldLocator {
        &self.review_identity_field
    }
    pub(in crate::domain_computation) const fn review_identity(&self) -> &AspectValue {
        &self.review_identity
    }
    pub(in crate::domain_computation) const fn review_type_field(&self) -> &AspectFieldLocator {
        &self.review_type_field
    }
    pub(in crate::domain_computation) const fn review_type(&self) -> &AspectValue {
        &self.review_type
    }
    pub(in crate::domain_computation) const fn review_status_field(&self) -> &AspectFieldLocator {
        &self.review_status_field
    }
    pub(in crate::domain_computation) const fn review_required_status(&self) -> &AspectValue {
        &self.review_required_status
    }
    pub(in crate::domain_computation) const fn requester_relation(&self) -> KindId {
        self.requester_relation
    }
    pub(in crate::domain_computation) const fn grant_relation(&self) -> KindId {
        self.grant_relation
    }
    pub(in crate::domain_computation) const fn resource_relation(&self) -> Option<KindId> {
        self.resource_relation
    }
    pub(in crate::domain_computation) const fn review_relation(&self) -> KindId {
        self.review_relation
    }
    pub(in crate::domain_computation) const fn review_scope_relation(&self) -> KindId {
        self.review_scope_relation
    }
    pub(in crate::domain_computation) fn required_program_targets(
        &self,
    ) -> &[ApplicationOperationProgramTarget] {
        &self.required_program_targets
    }
    pub(in crate::domain_computation) const fn lifecycle_effect(&self) -> Option<&worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect>{
        self.lifecycle_effect.as_ref()
    }
    pub(in crate::domain_computation) const fn requester(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.observed.upper_bound.requester()
    }

    pub(in crate::domain_computation) const fn resource(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.observed.upper_bound.resource()
    }

    pub(in crate::domain_computation) const fn grant(
        &self,
    ) -> worth_relational::facade::identity::EntityId {
        self.observed.upper_bound.grant()
    }

    pub(in crate::domain_computation) const fn upper_bound(
        &self,
    ) -> &WorthQueryElevationUpperBound {
        &self.observed.upper_bound
    }
    pub(in crate::domain_computation) const fn supporting(
        &self,
    ) -> &WorthQueryRetainedCapabilitySupport {
        &self.observed.supporting
    }
    pub(super) fn supporting_mut(&mut self) -> &mut WorthQueryRetainedCapabilitySupport {
        &mut self.observed.supporting
    }

    pub(in crate::domain_computation) fn apply_current_support<
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
