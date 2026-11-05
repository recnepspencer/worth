//! Moved whole from `domain_computation::provider_session::protocol::declared_closure::application`: the items below and every impl they own.
use crate::domain_computation::vocabulary::provider_session::declared_role_closure::WorthQueryProviderPlanDeclaredClosure;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationEffectPosture {
    Application,
    Platform,
    ApplicationWithPlatform,
}

impl WorthQueryApplicationEffectPosture {
    pub(in crate::domain_computation) const fn includes_platform(self) -> bool {
        !matches!(self, Self::Application)
    }

    pub(in crate::domain_computation::vocabulary::provider_session) const fn includes_application(
        self,
    ) -> bool {
        !matches!(self, Self::Platform)
    }
}

pub(in crate::domain_computation) fn bind_native_reference_invariants(
    closure: &mut WorthQueryProviderPlanDeclaredClosure,
    requirements: &mut Vec<
        worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement,
    >,
    native: &worth_query_installation::facade::WorthQueryInvariantExecutionContract,
) -> Result<(), &'static str> {
    for requirement in native.requirements() {
        if let Some(existing) = requirements
            .iter()
            .find(|existing| existing.slot() == requirement.slot())
        {
            if existing != requirement {
                return Err("native reference invariant conflicts with the installed application requirement");
            }
        } else {
            requirements.push(requirement.clone());
        }
        closure.invariant.push(requirement.slot().to_owned());
    }
    Ok(())
}

pub fn bind_application_candidate_effect_closure(
    closure: &mut WorthQueryProviderPlanDeclaredClosure,
    posture: WorthQueryApplicationEffectPosture,
    touches_application: bool,
    application_effects: &[String],
    application_invariants: &[String],
) {
    closure.effect.clear();
    closure.invariant.clear();
    if posture.includes_application() && touches_application {
        closure.effect.extend_from_slice(application_effects);
        closure.invariant.extend_from_slice(application_invariants);
    }
    if posture.includes_platform() {
        closure.effect.push("mutation".to_owned());
    }
}
