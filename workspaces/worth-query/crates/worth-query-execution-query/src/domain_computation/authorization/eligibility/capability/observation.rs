//! Select the head, admit the asker, then resolve the request and read
//! verdicts from admission's capability observation. Nothing here is
//! retained, reserved or written.
//!
//! The order is the ability eligibility disclosure rule: the asker is
//! admitted before any request resolves, and a roster's disclosure is
//! decided before the request resolves. Capability eligibility reads the
//! branch head only: grant validity is judged at the trusted instant of the
//! question, so a pinned commit's grants would be judged against a clock
//! that describes neither that commit nor now.

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::{
    ApplicationBindingContext, ApplicationOperationMarkerIdentity, ApplicationSchema,
    ApplicationStructuredValueBinding,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_execution_attempt::facade::ports::AuthenticatedPrincipalInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;

use super::super::prepared::{admit_asker_at, authorization, requirement_for};
use super::super::{
    WorthQueryAbilityEligibilityDenial as Denial, WorthQueryAbilityKey, WorthQueryEligibilityBasis,
};
use super::prepared::PreparedCapabilityEligibility;
use super::{WorthQueryCapabilityEligibility as Verdict, WorthQueryCapabilityEligibilityQuestion};
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::authorization::admission::admit_request;
use crate::domain_computation::authorization::capability_elevation_projection::validate_elevation_projection;
use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::preflight;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess;
use crate::domain_computation::primary_graph::resolution::WorthQueryCandidatePrincipalRefusal;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;

type Prepared<'a, 'runtime, Schema, Context, Capability, Input> = PreparedCapabilityEligibility<
    'a,
    'runtime,
    Schema,
    Context,
    <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    <Input as ApplicationCapabilityRequest<Schema, Capability>>::Context,
>;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Whether `asker` itself may exercise the question's capability at the
    /// head of `branch`: the outcome capability admission reaches for this
    /// principal on this request now. Asking about oneself needs no
    /// disclosure.
    pub fn observe_asker_capability_eligibility<Context, Capability, Operation, Input>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
        asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        question: &WorthQueryCapabilityEligibilityQuestion<
            Schema,
            Context,
            Capability,
            Operation,
            Input,
        >,
    ) -> Result<Verdict, Denial>
    where
        Context: ApplicationBindingContext<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        self.with_capability_question(phase, branch, request, question, |prepared, native| {
            // 1. Admit the asker at the head, before any request resolves.
            let admitted = admit_asker_at::<Schema, Context>(
                prepared.head,
                prepared.execution,
                prepared.binding,
                asker,
                request,
                native,
            );
            let admitted = match admitted {
                Ok(admitted) => admitted,
                Err(denial) => {
                    return WorthQueryCandidatePrincipalRefusal::of(denial.kind())
                        .map(Verdict::of_refusal)
                        .ok_or(Denial::Asker(denial));
                }
            };
            // 2. Resolve the request at the head and read the verdict.
            let head = prepared.resolve_on(native, prepared.head)?;
            prepared.verdict_for(native, &head, admitted.principal_entity_id())
        })
    }

    /// One verdict per candidate, in order, at the head of `branch`, behind
    /// the same disclosure gate as ability rosters: `asker` must be admitted
    /// and hold `disclosure` on the request's scope before any candidate is
    /// evaluated or the whole request resolves.
    pub fn observe_roster_capability_eligibility<Context, Capability, Operation, Input>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
        asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        question: &WorthQueryCapabilityEligibilityQuestion<
            Schema,
            Context,
            Capability,
            Operation,
            Input,
        >,
        disclosure: WorthQueryAbilityKey,
        candidates: &[WorthQueryExternalPrincipalIdentity],
    ) -> Result<Vec<Verdict>, Denial>
    where
        Context: ApplicationBindingContext<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        self.with_capability_question(phase, branch, request, question, |prepared, native| {
            // 1. Admit the asker at the head, before any request resolves.
            let asker = admit_asker_at::<Schema, Context>(
                prepared.head,
                prepared.execution,
                prepared.binding,
                asker,
                request,
                native,
            )
            .map_err(Denial::Asker)?;
            let disclosed = requirement_for::<Schema, Context>(self, prepared.binding, disclosure)?;
            // 2. Resolve only the head scope and 3. decide disclosure there.
            let scope = prepared.resolve_scope_on(native, prepared.head)?;
            let asker = asker.principal_entity_id();
            if !prepared.holds_disclosure(native, prepared.head, &scope, asker, disclosed)? {
                return Err(Denial::RosterNotDisclosed {
                    disclosure: disclosure.ability(),
                });
            }
            // 4. Only a disclosed roster resolves the whole request.
            let subject = prepared.resolve_on(native, prepared.head)?;
            let mut verdicts = Vec::with_capacity(candidates.len());
            for candidate in candidates {
                admit_request(request, prepared.installed.contract().operation())
                    .map_err(Denial::Authorization)?;
                verdicts.push(prepared.verdict_of(native, &subject, candidate)?);
            }
            Ok(verdicts)
        })
    }

    /// Selects the bases, the installed binding, capability plan and one
    /// trusted time sample, then lends them to `observe`. No request
    /// resolves here.
    fn with_capability_question<Context, Capability, Operation, Input, Answer>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: WorthQueryProductBranch,
        request: &WorthQueryRequestScope,
        question: &WorthQueryCapabilityEligibilityQuestion<
            Schema,
            Context,
            Capability,
            Operation,
            Input,
        >,
        observe: impl FnOnce(
            &Prepared<'_, '_, Schema, Context, Capability, Input>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<Answer, Denial>,
    ) -> Result<Answer, Denial>
    where
        Context: ApplicationBindingContext<Schema>,
        Operation: ApplicationOperationMarkerIdentity<Schema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
        Input: ApplicationCapabilityRequest<Schema, Capability> + 'static,
    {
        let key = question.capability();
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| authorization(AuthorizationRefusal::ForeignPhase, key.operation()))?;
        admit_request(request, key.operation()).map_err(Denial::Authorization)?;
        let (capability_ref, operation_ref) = question.references();
        let capability = self
            .installed_schema()
            .capability(capability_ref, operation_ref)
            .map_err(|_| Denial::CapabilityNotInstalled {
                capability: key.capability(),
            })?;
        let installed = preflight::admit_installed_plan(self, &capability, None)
            .map_err(Denial::Authorization)?;
        if installed.elevation().is_some() {
            return Err(Denial::CapabilityRequiresElevation {
                capability: key.capability(),
            });
        }
        validate_elevation_projection(installed.contract(), question.projection())
            .map_err(Denial::CapabilityRequest)?;
        let sample = preflight::sample_trusted_time(self, &capability, installed)
            .map_err(Denial::Authorization)?;
        let session =
            WorthQueryGraphWorkSessionIdentity::for_fact_free_observation().ok_or_else(|| {
                authorization(
                    AuthorizationRefusal::AdmissionIdentityExhausted,
                    key.operation(),
                )
            })?;
        let (head, _) =
            self.select_eligibility_bases(&execution, WorthQueryEligibilityBasis::Current(branch))?;
        let binding = self
            .installed_schema()
            .principal_binding(Context::principal_binding())
            .map_err(Denial::PrincipalBinding)?;
        // The capability's grantee is the principal kind it can admit.
        if installed.contract().delegation().grantee().from() != binding.principal_entity() {
            return Err(Denial::PrincipalEntityMismatch {
                ability: key.capability(),
            });
        }
        let prepared = PreparedCapabilityEligibility {
            runtime: self,
            execution: &execution,
            binding: &binding,
            binding_identity: self.installed_schema().binding_identity(),
            installed,
            capability_identity: *capability.identity().bytes(),
            projection: question.projection(),
            sample,
            session,
            head: &head,
        };
        observe(&prepared, &mut ApplicationQueryNativeAccess::acquire())
    }
}
