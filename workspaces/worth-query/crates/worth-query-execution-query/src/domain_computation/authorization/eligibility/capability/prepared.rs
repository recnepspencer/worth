//! The resolved parts one capability eligibility observation shares, and the
//! verdict for one principal: admission's own active-capability observation
//! run on a request captured for that principal, with its decision dropped.
//!
//! Requests resolve lazily, per basis, only after the steps that must precede
//! them have passed, so a refused asker learns nothing about which scopes
//! exist. One trusted time sample serves every verdict of one question.

use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestProjection;
use worth_query_declaration::facade::application_schema::{
    ApplicationBindingContext, ApplicationSchema,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledAbilityRequirement,
};
use worth_relational::facade::identity::EntityId;

use super::super::prepared::{
    authorization, decide_ability_on, foreign_runtime, InstalledBinding, PolicyDecisionSite,
};
use super::super::WorthQueryAbilityEligibilityDenial as Denial;
use super::source::FactFreeCapabilityObservation;
use super::WorthQueryCapabilityEligibility as Verdict;
use crate::domain_computation::authorization::capability_registry::WorthQueryInstalledCapabilityPlan;
use crate::domain_computation::authorization::delegation_admission::WorthQueryCapabilityObservationSource as _;
use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::observation::request_resolution::{
    resolve_capability_request, WorthQueryResolvedCapabilityRequest,
};
use crate::domain_computation::authorization::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as AuthorizationKind;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::authorization::WorthQueryRuntimeTimeSample;
use crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess;
use crate::domain_computation::primary_graph::{
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode,
    WorthQuerySelectedProductOperation,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;

/// One selected committed basis and the question's request resolved on it.
pub(super) struct CapabilityBasis<'a, 'runtime, Schema, Scope> {
    selected: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
    resolved: WorthQueryResolvedCapabilityRequest<Schema, Scope>,
}

pub(super) struct PreparedCapabilityEligibility<'a, 'runtime, Schema, Context, Scope, Slots>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
{
    pub(super) runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) execution: &'a worth_execution::ExecutionRequest<'a, 'a>,
    pub(super) binding: &'a InstalledBinding<Schema, Context>,
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) installed: &'a WorthQueryInstalledCapabilityPlan,
    pub(super) capability_identity: [u8; 32],
    pub(super) projection: &'a ApplicationCapabilityRequestProjection<Schema, Scope, Slots>,
    /// The one trusted instant every verdict of this question reads.
    pub(super) sample: WorthQueryRuntimeTimeSample,
    pub(super) session: WorthQueryGraphWorkSessionIdentity,
    /// Where the asker is admitted and the roster disclosed.
    pub(super) head: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
    /// Where verdicts are read; `None` reads them at the head.
    pub(super) pinned: Option<&'a WorthQuerySelectedProductOperation<'runtime, Schema>>,
}

impl<'a, 'runtime, Schema, Context, Scope, Slots>
    PreparedCapabilityEligibility<'a, 'runtime, Schema, Context, Scope, Slots>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
{
    fn name(&self) -> &str {
        self.installed.contract().name()
    }

    /// The question's request resolved on `selected`, exactly as admission
    /// resolves it.
    pub(super) fn resolve_on(
        &self,
        native: &mut ApplicationQueryNativeAccess<'_>,
        selected: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
    ) -> Result<CapabilityBasis<'a, 'runtime, Schema, Scope>, Denial> {
        let graph = self
            .runtime
            .runtime
            .primary_graph()
            .ok_or_else(|| authorization(AuthorizationRefusal::ForeignRuntime, self.name()))?;
        let resolution = graph.retain_entity_resolution_context();
        let snapshot = selected.application_basis().snapshot_handle().clone();
        let resolved = native
            .with_runtime(
                graph,
                self.execution,
                || foreign_runtime(self.name()),
                |relational| {
                    let truth = resolution
                        .at_snapshot(
                            relational,
                            &snapshot,
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .map_err(|_| projection_rejected(self.name()))?;
                    resolve_capability_request(
                        self.execution,
                        &truth,
                        self.runtime.installed_schema(),
                        self.projection,
                    )
                },
            )
            .map_err(Denial::CapabilityRequest)?;
        Ok(CapabilityBasis { selected, resolved })
    }

    /// Whether `principal` holds the installed `disclosure` ability on the
    /// resolved request's scope at `basis`.
    pub(super) fn holds_disclosure(
        &self,
        native: &mut ApplicationQueryNativeAccess<'_>,
        basis: &CapabilityBasis<'_, '_, Schema, Scope>,
        principal: EntityId,
        disclosure: &WorthQueryInstalledAbilityRequirement,
    ) -> Result<bool, Denial> {
        decide_ability_on(
            PolicyDecisionSite {
                runtime: self.runtime,
                execution: self.execution,
                binding_identity: &self.binding_identity,
                selected: basis.selected,
            },
            native,
            basis.resolved.resource(),
            principal,
            disclosure,
        )
    }

    /// The verdict for `identity` at `basis`: admission's refusal of the
    /// identity itself, or the capability observation's outcome.
    pub(super) fn verdict_of(
        &self,
        native: &mut ApplicationQueryNativeAccess<'_>,
        basis: &CapabilityBasis<'_, '_, Schema, Scope>,
        identity: &WorthQueryExternalPrincipalIdentity,
    ) -> Result<Verdict, Denial> {
        let principal = basis
            .selected
            .resolve_candidate_principal_with_native_access(
                self.execution,
                self.binding,
                identity,
                native,
            )
            .map_err(Denial::Candidate)?;
        match principal {
            Ok(principal) => self.verdict_for(native, basis, principal),
            Err(refusal) => Ok(Verdict::of_refusal(refusal)),
        }
    }

    /// Admission's active-capability observation for `principal` at `basis`.
    /// A policy refusal is the verdict; any other stop refuses the question.
    pub(super) fn verdict_for(
        &self,
        native: &mut ApplicationQueryNativeAccess<'_>,
        basis: &CapabilityBasis<'_, '_, Schema, Scope>,
        principal: EntityId,
    ) -> Result<Verdict, Denial> {
        let request = WorthQueryRetainedCapabilityRequest::capture(
            self.capability_identity,
            principal,
            self.projection,
            &basis.resolved,
        );
        CapabilityVerdictSite {
            runtime: self.runtime,
            execution: self.execution,
            installed: self.installed,
            sample: &self.sample,
            session: self.session,
        }
        .verdict(native, basis.selected, &request)
    }
}

/// What one capability verdict reads besides its request and basis.
pub(in super::super) struct CapabilityVerdictSite<'a, Schema> {
    pub(in super::super) runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(in super::super) execution: &'a worth_execution::ExecutionRequest<'a, 'a>,
    pub(in super::super) installed: &'a WorthQueryInstalledCapabilityPlan,
    pub(in super::super) sample: &'a WorthQueryRuntimeTimeSample,
    pub(in super::super) session: WorthQueryGraphWorkSessionIdentity,
}

impl<Schema: ApplicationSchema> CapabilityVerdictSite<'_, Schema> {
    /// Admission's active-capability observation of `request` at `selected`.
    /// A policy refusal is the verdict; any other stop refuses the question.
    pub(in super::super) fn verdict(
        &self,
        native: &mut ApplicationQueryNativeAccess<'_>,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        request: &WorthQueryRetainedCapabilityRequest,
    ) -> Result<Verdict, Denial> {
        let name = self.installed.contract().name();
        let graph = self
            .runtime
            .runtime
            .primary_graph()
            .ok_or_else(|| authorization(AuthorizationRefusal::ForeignRuntime, name))?;
        let snapshot = selected.application_basis().snapshot_handle().clone();
        let observed = native.with_runtime(
            graph,
            self.execution,
            || foreign_runtime(name),
            |relational| {
                FactFreeCapabilityObservation {
                    session: self.session,
                    relational,
                    snapshot: &snapshot,
                    bridge: self.runtime.runtime.authorization.bridge(),
                    installed: self.installed,
                    request,
                    sample: self.sample,
                }
                .observe_active_capability(self.execution, None, None)
                // The decision is read for its outcome and dropped here.
                .map(drop)
            },
        );
        match observed {
            Ok(()) => Ok(Verdict::Eligible),
            Err(denial) => match denial.explanation_cause() {
                Some(cause) => Ok(Verdict::Denied(cause)),
                None => Err(Denial::Authorization(denial)),
            },
        }
    }
}

fn projection_rejected(subject: &str) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        AuthorizationKind::Refused(AuthorizationRefusal::CapabilityProjectionRejected),
        subject,
    )
}
