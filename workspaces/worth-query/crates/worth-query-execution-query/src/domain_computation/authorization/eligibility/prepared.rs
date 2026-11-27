//! The resolved parts one eligibility observation shares.
//!
//! The asker is always admitted, and the roster disclosed, at the branch
//! head: a pin selects what is reported, never who may ask. Scopes resolve
//! lazily, only after the steps that must precede them have passed, so a
//! refused asker learns nothing about which scopes exist. Verdicts read the
//! subject basis, which is the head unless a retained observation pins it.

use super::WorthQueryAbilityKey;
use super::{
    WorthQueryAbilityEligibility as Verdict, WorthQueryAbilityEligibilityDenial as Denial,
};
use crate::domain_computation::authorization::admission::policy_decision::{
    WorthQueryInstalledPolicyDecision, WorthQueryInstalledPolicySubject,
};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as AuthorizationKind;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionDenial,
    WorthQueryPrincipalResolutionMode, WorthQuerySelectedProductOperation,
};
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::{
    ApplicationBindingContext, ApplicationSchema,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledAbilityRequirement,
    WorthQueryInstalledPrincipalBinding,
};
use worth_relational::facade::identity::EntityId;

pub(super) type InstalledBinding<Schema, Context> = WorthQueryInstalledPrincipalBinding<
    Schema,
    <Context as ApplicationBindingContext<Schema>>::PrincipalBinding,
    <Context as ApplicationBindingContext<Schema>>::Mapping,
    <Context as ApplicationBindingContext<Schema>>::Principal,
    <Context as ApplicationBindingContext<Schema>>::PrincipalIdentity,
    <Context as ApplicationBindingContext<Schema>>::PrincipalIdentityBinding,
>;

pub(super) type ContextPrincipal<Schema, Context> = WorthQueryAuthenticatedPrincipal<
    Schema,
    <Context as ApplicationBindingContext<Schema>>::Principal,
    <Context as ApplicationBindingContext<Schema>>::PrincipalIdentity,
>;

/// One selected committed basis and the question's scope resolved on it.
pub(super) struct SelectedBasis<'a, 'runtime, Schema, Scope> {
    selected: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
    scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
}

pub(super) struct PreparedEligibility<'a, 'runtime, Schema: ApplicationSchema, Context, Field>
where
    Context: ApplicationBindingContext<Schema>,
    Field: RecordSelection,
{
    pub(super) runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) execution: &'a worth_execution::ExecutionRequest<'a, 'a>,
    pub(super) request: &'a WorthQueryRequestScope,
    pub(super) binding: &'a InstalledBinding<Schema, Context>,
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) requirement: &'a WorthQueryInstalledAbilityRequirement,
    /// Where the asker is admitted and the roster disclosed.
    pub(super) head: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
    /// Where verdicts are read; `None` reads them at the head.
    pub(super) pinned: Option<&'a WorthQuerySelectedProductOperation<'runtime, Schema>>,
    /// The question's scope selection, resolved per basis when reached.
    pub(super) field: Field,
    pub(super) value: Field::Value,
}

impl<'a, 'runtime, Schema, Context, Field> PreparedEligibility<'a, 'runtime, Schema, Context, Field>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
    Field: RecordSelection,
    Field::Value: Clone,
{
    /// The question's scope resolved on `selected`.
    pub(super) fn basis_on(
        &self,
        native: &mut ApplicationQueryNativeAccess<'_>,
        selected: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
    ) -> Result<SelectedBasis<'a, 'runtime, Schema, Field::Entity>, Denial> {
        let scope = selected
            .resolve_entity_admitted_with_native_access(
                self.execution,
                self.field,
                self.value.clone(),
                self.request,
                native,
            )
            .map_err(Denial::Scope)?;
        Ok(SelectedBasis { selected, scope })
    }

    pub(super) fn requirement_of(
        &self,
        ability: WorthQueryAbilityKey,
    ) -> Result<&'a WorthQueryInstalledAbilityRequirement, Denial> {
        requirement_for::<Schema, Context>(self.runtime, self.binding, ability)
    }

    /// The asker, admitted at the head exactly as a request admits it.
    pub(super) fn resolve_asker(
        &self,
        native: &mut ApplicationQueryNativeAccess<'_>,
        asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    ) -> Result<ContextPrincipal<Schema, Context>, WorthQueryPrincipalResolutionDenial> {
        admit_asker_at::<Schema, Context>(
            self.head,
            self.execution,
            self.binding,
            asker,
            self.request,
            native,
        )
    }

    /// The verdict for `identity` at the subject basis: admission's refusal
    /// of the identity itself, or the installed policy decision.
    pub(super) fn verdict_of(
        &self,
        native: &mut ApplicationQueryNativeAccess<'_>,
        subject: &SelectedBasis<'_, '_, Schema, Field::Entity>,
        identity: &WorthQueryExternalPrincipalIdentity,
    ) -> Result<Verdict, Denial> {
        let principal = subject
            .selected
            .resolve_candidate_principal_with_native_access(
                self.execution,
                self.binding,
                identity,
                native,
            )
            .map_err(Denial::Candidate)?;
        match principal {
            Ok(principal) => self
                .decide(native, subject, principal, self.requirement)
                .map(Verdict::of_policy),
            Err(refusal) => Ok(Verdict::of_refusal(refusal)),
        }
    }

    /// The installed policy decision for one principal entity on `basis`.
    pub(super) fn decide(
        &self,
        native: &mut ApplicationQueryNativeAccess<'_>,
        basis: &SelectedBasis<'_, '_, Schema, Field::Entity>,
        principal_entity_id: EntityId,
        requirement: &WorthQueryInstalledAbilityRequirement,
    ) -> Result<bool, Denial> {
        decide_ability_on(
            PolicyDecisionSite {
                runtime: self.runtime,
                execution: self.execution,
                binding_identity: &self.binding_identity,
                selected: basis.selected,
            },
            native,
            &basis.scope,
            principal_entity_id,
            requirement,
        )
    }
}

/// The asker admitted at `head` exactly as a request admits it.
pub(super) fn admit_asker_at<Schema, Context>(
    head: &WorthQuerySelectedProductOperation<'_, Schema>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    binding: &InstalledBinding<Schema, Context>,
    asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
    request: &WorthQueryRequestScope,
    native: &mut ApplicationQueryNativeAccess<'_>,
) -> Result<ContextPrincipal<Schema, Context>, WorthQueryPrincipalResolutionDenial>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
{
    head.resolve_authenticated_principal_admitted_with_native_access(
        execution,
        binding,
        asker,
        request,
        WorthQueryPrincipalResolutionMode::Ordinary,
        None,
        native,
    )
}

/// The runtime, request and committed basis one policy decision reads.
pub(super) struct PolicyDecisionSite<'a, 'runtime, Schema> {
    pub(super) runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) execution: &'a worth_execution::ExecutionRequest<'a, 'a>,
    pub(super) binding_identity: &'a ApplicationSchemaBindingIdentity,
    pub(super) selected: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
}

/// The installed ability decision for one principal entity on `scope` at the
/// site's basis: the evaluator admission uses, with no decision fact minted.
pub(super) fn decide_ability_on<Schema: ApplicationSchema, Scope>(
    site: PolicyDecisionSite<'_, '_, Schema>,
    native: &mut ApplicationQueryNativeAccess<'_>,
    scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
    principal_entity_id: EntityId,
    requirement: &WorthQueryInstalledAbilityRequirement,
) -> Result<bool, Denial> {
    observe_ability_on(site, native, scope, principal_entity_id, requirement)
        .map(|decision| decision.is_allowed())
}

/// [`decide_ability_on`], keeping the decision's evidence: which installed
/// policy paths held.
pub(super) fn observe_ability_on<Schema: ApplicationSchema, Scope>(
    site: PolicyDecisionSite<'_, '_, Schema>,
    native: &mut ApplicationQueryNativeAccess<'_>,
    scope: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
    principal_entity_id: EntityId,
    requirement: &WorthQueryInstalledAbilityRequirement,
) -> Result<WorthQueryInstalledPolicyDecision, Denial> {
    let graph =
        site.runtime.runtime.primary_graph().ok_or_else(|| {
            authorization(AuthorizationRefusal::ForeignRuntime, requirement.policy())
        })?;
    let snapshot = site.selected.application_basis().snapshot_handle().clone();
    native
        .with_runtime(
            graph,
            site.execution,
            || foreign_runtime(requirement.policy()),
            |relational| {
                WorthQueryInstalledPolicySubject {
                    relational,
                    snapshot,
                    principal_entity_id,
                    scope_identity: scope,
                    binding_identity: site.binding_identity,
                }
                .decide(site.execution, site.runtime, requirement)
            },
        )
        .map_err(Denial::Authorization)
}

/// The installed requirement for `ability`, whose policy paths must all start
/// at the binding's principal entity.
pub(super) fn requirement_for<'a, Schema, Context>(
    runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    binding: &InstalledBinding<Schema, Context>,
    ability: WorthQueryAbilityKey,
) -> Result<&'a WorthQueryInstalledAbilityRequirement, Denial>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
{
    let requirement = runtime
        .installed_schema()
        .installed_ability_requirement(ability.ability(), ability.scope_entity())
        .ok_or(Denial::AbilityNotInstalled {
            ability: ability.ability(),
        })?;
    let paths = requirement.policy_paths();
    let starts_at_principal = !paths.is_empty()
        && paths
            .iter()
            .all(|installed| installed.path().principal_entity() == binding.principal_entity());
    if starts_at_principal {
        Ok(requirement)
    } else {
        Err(Denial::PrincipalEntityMismatch {
            ability: ability.ability(),
        })
    }
}

pub(super) fn foreign_runtime(subject: &str) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        AuthorizationKind::Refused(AuthorizationRefusal::ForeignRuntime),
        subject,
    )
}

pub(super) fn authorization(refusal: AuthorizationRefusal, subject: &str) -> Denial {
    Denial::Authorization(WorthQueryOperationAuthorizationDenial::new(
        AuthorizationKind::Refused(refusal),
        subject,
    ))
}
