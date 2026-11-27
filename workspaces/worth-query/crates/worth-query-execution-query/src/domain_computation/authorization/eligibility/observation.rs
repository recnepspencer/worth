//! Select the bases, resolve the scope and principals, then read verdicts
//! from the installed policy decision. Nothing here mints, reserves or writes.

use super::prepared::{authorization, requirement_for, PreparedEligibility, SelectedBasis};
use super::{
    WorthQueryAbilityEligibility as Verdict, WorthQueryAbilityEligibilityDenial as Denial,
    WorthQueryAbilityKey, WorthQueryEligibilityBasis, WorthQueryEligibilityQuestion,
};
use crate::domain_computation::authorization::admission::admit_request;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess;
use crate::domain_computation::primary_graph::resolution::WorthQueryCandidatePrincipalRefusal;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQuerySelectedProductOperation,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::{
    ApplicationBindingContext, ApplicationSchema,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_execution_attempt::facade::ports::AuthenticatedPrincipalInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchInternalPort as _;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Whether `asker` itself may exercise the question's ability at `basis`.
    ///
    /// The verdict is the installed policy decision admission reaches for this
    /// principal there. Asking about oneself needs no disclosure, but asking
    /// at a retained basis needs the asker admitted at the head. No decision
    /// fact is minted and no commit, grant or ledger record results.
    pub fn observe_asker_eligibility<Context, Ability, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        basis: WorthQueryEligibilityBasis<'_>,
        asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
    ) -> Result<Verdict, Denial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        self.with_eligibility_question(phase, basis, request, question, |prepared, native| {
            let admitted = prepared.resolve_asker(native, asker);
            if prepared.pinned.is_some() {
                admitted.map_err(Denial::Asker)?;
                return prepared.verdict_of(native, asker.identity());
            }
            match admitted {
                Ok(asker) => {
                    let asker = asker.principal_entity_id();
                    prepared
                        .decide(native, &prepared.head, asker, prepared.requirement)
                        .map(Verdict::of_policy)
                }
                Err(denial) => WorthQueryCandidatePrincipalRefusal::of(denial.kind())
                    .map(Verdict::of_refusal)
                    .ok_or(Denial::Asker(denial)),
            }
        })
    }

    /// One verdict per candidate, in order, at `basis`.
    ///
    /// Revealing another principal's authority is disclosure: `asker` must be
    /// admitted and hold `disclosure` on the same scope at the head, decided
    /// by the same fact-free observation, before any candidate is evaluated.
    pub fn observe_roster_eligibility<Context, Ability, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        basis: WorthQueryEligibilityBasis<'_>,
        asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
        disclosure: WorthQueryAbilityKey,
        candidates: &[WorthQueryExternalPrincipalIdentity],
    ) -> Result<Vec<Verdict>, Denial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        self.with_eligibility_question(phase, basis, request, question, |prepared, native| {
            let asker = prepared
                .resolve_asker(native, asker)
                .map_err(Denial::Asker)?;
            let disclosed = prepared.requirement_of(disclosure)?;
            let asker = asker.principal_entity_id();
            if !prepared.decide(native, &prepared.head, asker, disclosed)? {
                return Err(Denial::RosterNotDisclosed {
                    disclosure: disclosure.ability(),
                });
            }
            let mut verdicts = Vec::with_capacity(candidates.len());
            for candidate in candidates {
                admit_request(request, prepared.requirement.ability())
                    .map_err(Denial::Authorization)?;
                verdicts.push(prepared.verdict_of(native, candidate)?);
            }
            Ok(verdicts)
        })
    }

    /// Selects the head (and any pinned basis), the installed binding and
    /// requirement, and the scope on each basis, then lends them to `observe`.
    fn with_eligibility_question<Context, Ability, Field, Answer>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        basis: WorthQueryEligibilityBasis<'_>,
        request: &WorthQueryRequestScope,
        question: &WorthQueryEligibilityQuestion<Schema, Context, Ability, Field>,
        observe: impl FnOnce(
            &PreparedEligibility<'_, '_, Schema, Context, Field::Entity>,
            &mut ApplicationQueryNativeAccess<'_>,
        ) -> Result<Answer, Denial>,
    ) -> Result<Answer, Denial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        let ability = question.ability();
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| authorization(AuthorizationRefusal::ForeignPhase, ability.ability()))?;
        admit_request(request, ability.ability()).map_err(Denial::Authorization)?;
        let (head, pinned) = self.select_eligibility_bases(&execution, basis)?;
        let binding = self
            .installed_schema()
            .principal_binding(Context::principal_binding())
            .map_err(Denial::PrincipalBinding)?;
        let requirement = requirement_for::<Schema, Context>(self, &binding, ability)?;
        let mut native = ApplicationQueryNativeAccess::acquire();
        let mut resolve_scope = |selected: &WorthQuerySelectedProductOperation<'_, Schema>| {
            let (field, value) = question.scope_selection();
            selected
                .resolve_entity_admitted_with_native_access(
                    &execution,
                    field,
                    value,
                    request,
                    &mut native,
                )
                .map_err(Denial::Scope)
        };
        let head_scope = resolve_scope(&head)?;
        let pinned_scope = pinned.as_ref().map(&mut resolve_scope).transpose()?;
        let prepared = PreparedEligibility {
            runtime: self,
            execution: &execution,
            request,
            binding: &binding,
            binding_identity: self.installed_schema().binding_identity(),
            requirement,
            head: SelectedBasis {
                selected: &head,
                scope: &head_scope,
            },
            pinned: pinned
                .as_ref()
                .zip(pinned_scope.as_ref())
                .map(|(selected, scope)| SelectedBasis { selected, scope }),
        };
        observe(&prepared, &mut native)
    }

    /// The branch head, where the asker is admitted, and the retained basis
    /// verdicts read when one is pinned.
    fn select_eligibility_bases(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: WorthQueryEligibilityBasis<'_>,
    ) -> Result<
        (
            WorthQuerySelectedProductOperation<'_, Schema>,
            Option<WorthQuerySelectedProductOperation<'_, Schema>>,
        ),
        Denial,
    > {
        let (branch, retained) = match basis {
            WorthQueryEligibilityBasis::Current(branch) => (branch, None),
            WorthQueryEligibilityBasis::Retained(observation) => (
                crate::basis::WorthQueryProductBranch::from_occurrence(
                    observation.branch_incarnation(),
                ),
                Some(observation),
            ),
        };
        let head = self
            .on_branch(branch)
            .select_with_request(execution)
            .map_err(Denial::Basis)?;
        let pinned = retained
            .map(|observation| {
                self.select_application_read_observation_with_request(execution, observation)
            })
            .transpose()
            .map_err(Denial::Basis)?;
        Ok((head, pinned))
    }
}
