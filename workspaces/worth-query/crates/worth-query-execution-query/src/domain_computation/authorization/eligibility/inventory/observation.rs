//! Admit the asker and decide disclosure at the head, then resolve the
//! subject and read every installed authority on the scope there (or at a
//! pinned basis). The disclosure order is the roster's: nothing about the
//! subject or the pinned scope is read before disclosure is decided.

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::fragment::RecordSelection;
use worth_query_declaration::facade::application_schema::{
    ApplicationAuthorizationPath, ApplicationBindingContext, ApplicationSchema,
};
use worth_query_execution_attempt::facade::ports::AuthenticatedPrincipalInternalPort as _;
use worth_query_execution_contracts::facade::runtime_time::WorthQueryRuntimeTimeReading;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_installation::facade::WorthQueryInstalledAbilityRequirement;
use worth_relational::facade::identity::EntityId;

use super::super::capability::prepared::CapabilityVerdictSite;
use super::super::prepared::{
    admit_asker_at, authorization, observe_ability_on, requirement_for, InstalledBinding,
    PolicyDecisionSite,
};
use super::super::{
    WorthQueryAbilityEligibility as Standing, WorthQueryAbilityEligibilityDenial as Denial,
    WorthQueryCapabilityEligibility, WorthQueryEligibilityBasis,
};
use super::{
    WorthQueryAuthority, WorthQueryAuthorityEntry, WorthQueryAuthorityInventory,
    WorthQueryAuthorityPath, WorthQueryAuthorityQuestion, WorthQueryAuthorityVerdict as Verdict,
};
use crate::domain_computation::authorization::admission::admit_request;
use crate::domain_computation::authorization::capability_registry::WorthQueryInstalledCapabilityPlan;
use crate::domain_computation::authorization::operation_admission::capability_admission::preparation::preflight::bridge_matches_plan;
use crate::domain_computation::authorization::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryApplicationEntityIdentity,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
};
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity;

/// The subject's basis: the selected operation and the scope resolved on it.
struct Subject<'a, 'runtime, Schema, Scope> {
    selected: &'a WorthQuerySelectedProductOperation<'runtime, Schema>,
    scope: WorthQueryApplicationEntityIdentity<Schema, Scope>,
    principal: EntityId,
}

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Every installed ability and capability the question's subject may
    /// exercise on its scope at `basis`, behind the asker's disclosure.
    pub fn observe_authority_inventory<Context, Field>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        basis: WorthQueryEligibilityBasis<'_>,
        asker: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        question: &WorthQueryAuthorityQuestion<Schema, Context, Field>,
    ) -> Result<WorthQueryAuthorityInventory, Denial>
    where
        Context: ApplicationBindingContext<Schema>,
        Field: RecordSelection,
        Field::Value: Clone,
    {
        let (field, value) = question.scope_selection();
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| authorization(AuthorizationRefusal::ForeignPhase, field.entity()))?;
        admit_request(request, field.entity()).map_err(Denial::Authorization)?;
        let (head, pinned) = self.select_eligibility_bases(&execution, basis)?;
        let binding = self
            .installed_schema()
            .principal_binding(Context::principal_binding())
            .map_err(Denial::PrincipalBinding)?;
        let disclosure = requirement_for::<Schema, Context>(self, &binding, question.disclosure())?;
        let native = &mut ApplicationQueryNativeAccess::acquire();
        // 1. Admit the asker and 2. decide disclosure on the head scope.
        let asker =
            admit_asker_at::<Schema, Context>(&head, &execution, &binding, asker, request, native)
                .map_err(Denial::Asker)?;
        let resolve = |selected: &WorthQuerySelectedProductOperation<'_, Schema>,
                       native: &mut ApplicationQueryNativeAccess<'_>| {
            selected
                .resolve_entity_admitted_with_native_access(
                    &execution,
                    field,
                    value.clone(),
                    request,
                    native,
                )
                .map_err(Denial::Scope)
        };
        let head_scope = resolve(&head, native)?;
        let binding_identity = self.installed_schema().binding_identity();
        let site = |selected| PolicyDecisionSite {
            runtime: self,
            execution: &execution,
            binding_identity: &binding_identity,
            selected,
        };
        let disclosed = observe_ability_on(
            site(&head),
            native,
            &head_scope,
            asker.principal_entity_id(),
            disclosure,
        )?;
        if !disclosed.is_allowed() {
            return Err(Denial::RosterNotDisclosed {
                disclosure: question.disclosure().ability(),
            });
        }
        // 3. Only a disclosed inventory resolves the subject (and any pin).
        let selected = pinned.as_ref().unwrap_or(&head);
        let scope = match pinned.as_ref() {
            Some(pinned) => resolve(pinned, native)?,
            None => head_scope,
        };
        let principal = selected
            .resolve_candidate_principal_with_native_access(
                &execution,
                &binding,
                question.subject(),
                native,
            )
            .map_err(Denial::Candidate)?;
        let principal = match principal {
            Ok(principal) => principal,
            Err(refusal) => {
                let standing = Standing::of_refusal(refusal);
                return Ok(WorthQueryAuthorityInventory::new(
                    standing,
                    field.entity(),
                    Vec::new(),
                ));
            }
        };
        let subject = Subject {
            selected,
            scope,
            principal,
        };
        let mut entries = Vec::new();
        // One trusted instant judges every capability of this inventory.
        let mut instant = None;
        for requirement in self
            .installed_schema()
            .installed_ability_requirements_on(field.entity())
        {
            if !starts_at::<Schema, Context>(requirement, &binding) {
                continue;
            }
            let decision = observe_ability_on(
                site(selected),
                native,
                &subject.scope,
                principal,
                requirement,
            )?;
            entries.push(WorthQueryAuthorityEntry {
                authority: WorthQueryAuthority::Ability {
                    ability: requirement.ability().to_owned(),
                    policy: requirement.policy().to_owned(),
                },
                verdict: ability_verdict(requirement, decision),
            });
        }
        for (identity, installed) in self.runtime.authorization.capability_plans() {
            let contract = installed.contract();
            if installed.request().resource_entity != field.entity()
                || contract.delegation().grantee().from() != binding.principal_entity()
                || !bridge_matches_plan(self, &binding_identity, installed)
            {
                continue;
            }
            let verdict = if installed.has_fixed_target() {
                let reading = match instant.take() {
                    Some(reading) => reading,
                    None => self.trusted_instant(installed)?,
                };
                let verdict = self.capability_verdict(
                    &execution, native, &subject, &reading, *identity, installed,
                )?;
                instant = Some(reading);
                verdict
            } else {
                Verdict::RequiresRequest
            };
            entries.push(WorthQueryAuthorityEntry {
                authority: WorthQueryAuthority::Capability {
                    capability: contract.name().to_owned(),
                    operation: contract.operation().to_owned(),
                },
                verdict,
            });
        }
        Ok(WorthQueryAuthorityInventory::new(
            Standing::Eligible,
            field.entity(),
            entries,
        ))
    }

    /// The inventory's one trusted reading, taken when its first capability
    /// needs a verdict.
    fn trusted_instant(
        &self,
        installed: &WorthQueryInstalledCapabilityPlan,
    ) -> Result<WorthQueryRuntimeTimeReading<'_>, Denial> {
        self.runtime.authorization_clock.read().map_err(|_| {
            authorization(
                AuthorizationRefusal::TrustedTimeUnavailable,
                installed.contract().name(),
            )
        })
    }

    /// A fixed-target capability's verdict: admission's observation of its
    /// own target request for the subject, at the inventory's instant.
    fn capability_verdict<Scope>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &mut ApplicationQueryNativeAccess<'_>,
        subject: &Subject<'_, '_, Schema, Scope>,
        reading: &WorthQueryRuntimeTimeReading<'_>,
        identity: [u8; 32],
        installed: &WorthQueryInstalledCapabilityPlan,
    ) -> Result<Verdict, Denial> {
        let name = installed.contract().name();
        let sample = reading
            .sample(installed.request().timeline)
            .map_err(|_| authorization(AuthorizationRefusal::TrustedTimeUnavailable, name))?;
        let session = WorthQueryGraphWorkSessionIdentity::for_fact_free_observation()
            .ok_or_else(|| authorization(AuthorizationRefusal::AdmissionIdentityExhausted, name))?;
        let request = WorthQueryRetainedCapabilityRequest::for_installed_target(
            identity,
            subject.principal,
            subject.scope.entity_id(),
            installed,
        );
        let site = CapabilityVerdictSite {
            runtime: self,
            execution,
            installed,
            sample: &sample,
            session,
        };
        Ok(match site.verdict(native, subject.selected, &request)? {
            WorthQueryCapabilityEligibility::Eligible => Verdict::Allowed {
                through: vec![WorthQueryAuthorityPath {
                    relations: vec![
                        installed
                            .contract()
                            .delegation()
                            .grantee()
                            .relation()
                            .to_owned(),
                        installed
                            .contract()
                            .target()
                            .resource()
                            .relation()
                            .to_owned(),
                    ],
                }],
            },
            WorthQueryCapabilityEligibility::Denied(cause) => Verdict::Denied(Some(cause)),
            // A resolved principal is never a candidate refusal here.
            _ => Verdict::Denied(None),
        })
    }
}

/// Whether every policy path of `requirement` starts at the binding's
/// principal entity: the abilities this principal binding can hold.
fn starts_at<Schema, Context>(
    requirement: &WorthQueryInstalledAbilityRequirement,
    binding: &InstalledBinding<Schema, Context>,
) -> bool
where
    Schema: ApplicationSchema,
    Context: ApplicationBindingContext<Schema>,
{
    let paths = requirement.policy_paths();
    !paths.is_empty()
        && paths
            .iter()
            .all(|installed| installed.path().principal_entity() == binding.principal_entity())
}

/// The ability verdict, naming the installed allowing paths that held.
fn ability_verdict(
    requirement: &WorthQueryInstalledAbilityRequirement,
    decision: crate::domain_computation::authorization::admission::policy_decision::WorthQueryInstalledPolicyDecision,
) -> Verdict {
    if !decision.is_allowed() {
        return Verdict::Denied(None);
    }
    let (relational, _) = decision.into_evidence();
    let held = relational.paths().iter().map(|path| path.matched());
    let mut through: Vec<WorthQueryAuthorityPath> = Vec::new();
    // Several rules may allow over the same relation chain; each chain is
    // one path, first-held order kept.
    for (installed, _) in requirement
        .policy_paths()
        .iter()
        .zip(held)
        .filter(|(installed, held)| *held && allows(installed.path()))
    {
        let path = WorthQueryAuthorityPath {
            relations: installed
                .path()
                .traversals()
                .iter()
                .map(|traversal| traversal.relation().to_owned())
                .collect(),
        };
        if !through.contains(&path) {
            through.push(path);
        }
    }
    Verdict::Allowed { through }
}

fn allows(path: &ApplicationAuthorizationPath) -> bool {
    matches!(
        path.effect(),
        worth_query_declaration::facade::application_schema::ApplicationAuthorizationPathEffect::Allow
    )
}
