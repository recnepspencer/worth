//! The resolved parts one eligibility observation shares.
//!
//! The asker is always admitted, and the roster disclosed, at the branch
//! head: a pin selects what is reported, never who may ask. Verdicts read
//! the subject basis, which is the head unless a retained observation pins it.

use super::WorthQueryAbilityKey;
use super::{
    WorthQueryAbilityEligibility as Verdict, WorthQueryAbilityEligibilityDenial as Denial,
};
use crate::domain_computation::authorization::admission::policy_decision::WorthQueryInstalledPolicySubject;
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

type ContextPrincipal<Schema, Context> = WorthQueryAuthenticatedPrincipal<
    Schema,
    <Context as ApplicationBindingContext<Schema>>::Principal,
    <Context as ApplicationBindingContext<Schema>>::PrincipalIdentity,
>;

/// One selected committed basis and the question's scope resolved on it.
pub(super) struct SelectedBasis<'a, 'runtime, Schema, Scope> {
    pub(super) selected: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
    pub(super) scope: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
}

pub(super) struct PreparedEligibility<'a, 'runtime, Schema: ApplicationSchema, Context, Scope>
where
    Context: ApplicationBindingContext<Schema>,
{
    pub(super) runtime: &'a WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    pub(super) execution: &'a worth_execution::ExecutionRequest<'a, 'a>,
    pub(super) request: &'a WorthQueryRequestScope,
    pub(super) binding: &'a InstalledBinding<Schema, Context>,
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) requirement: &'a WorthQueryInstalledAbilityRequirement,
    /// Where the asker is admitted and the roster disclosed.
    pub(super) head: SelectedBasis<'a, 'runtime, Schema, Scope>,
    /// Where verdicts are read; `None` reads them at the head.
    pub(super) pinned: Option<SelectedBasis<'a, 'runtime, Schema, Scope>>,
}

impl<'a, Schema, Context, Scope> PreparedEligibility<'a, '_, Schema, Context, Scope>
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
{
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
        self.head
            .selected
            .resolve_authenticated_principal_admitted_with_native_access(
                self.execution,
                self.binding,
                asker,
                self.request,
                WorthQueryPrincipalResolutionMode::Ordinary,
                None,
                native,
            )
    }

    /// The verdict for `identity` at the subject basis: admission's refusal
    /// of the identity itself, or the installed policy decision.
    pub(super) fn verdict_of(
        &self,
        native: &mut ApplicationQueryNativeAccess<'_>,
        identity: &WorthQueryExternalPrincipalIdentity,
    ) -> Result<Verdict, Denial> {
        let subject = self.pinned.as_ref().unwrap_or(&self.head);
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
        basis: &SelectedBasis<'_, '_, Schema, Scope>,
        principal_entity_id: EntityId,
        requirement: &WorthQueryInstalledAbilityRequirement,
    ) -> Result<bool, Denial> {
        let graph = self.runtime.runtime.primary_graph().ok_or_else(|| {
            authorization(AuthorizationRefusal::ForeignRuntime, requirement.policy())
        })?;
        let snapshot = basis.selected.application_basis().snapshot_handle().clone();
        native
            .with_runtime(
                graph,
                self.execution,
                || foreign_runtime(requirement.policy()),
                |relational| {
                    WorthQueryInstalledPolicySubject {
                        relational,
                        snapshot,
                        principal_entity_id,
                        scope_identity: basis.scope,
                        binding_identity: &self.binding_identity,
                    }
                    .decide(self.execution, self.runtime, requirement)
                },
            )
            .map(|decision| decision.is_allowed())
            .map_err(Denial::Authorization)
    }
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

fn foreign_runtime(subject: &str) -> WorthQueryOperationAuthorizationDenial {
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
