//! One provider role's declared closure: the reads, touches, effects and invariants it binds.
use worth_query_installation::facade::WorthQueryOperationEffectContract;
use worth_query_installation::facade::WorthQueryOperationGraphAccess;
use worth_query_installation::facade::WorthQueryOperationInvariantContract;

#[derive(Clone, Debug, Default)]
pub struct WorthQueryProviderPlanDeclaredClosure {
    pub(in crate::domain_computation) read: Vec<String>,
    pub(in crate::domain_computation) touch: Vec<String>,
    pub(in crate::domain_computation) effect: Vec<String>,
    pub(in crate::domain_computation) invariant: Vec<String>,
}

impl WorthQueryProviderPlanDeclaredClosure {
    pub(in crate::domain_computation) fn canonicalize(&mut self) {
        for values in [
            &mut self.read,
            &mut self.touch,
            &mut self.effect,
            &mut self.invariant,
        ] {
            values.sort();
            values.dedup();
        }
    }
}

pub fn bind_direct_role_closure(
    closure: &mut WorthQueryProviderPlanDeclaredClosure,
    effects: &[String],
    invariants: &[String],
) {
    if !closure.touch.is_empty() {
        closure.effect.clear();
        closure.effect.extend_from_slice(effects);
    }
    closure.invariant.clear();
    closure.invariant.extend_from_slice(invariants);
    closure.canonicalize();
}

pub(in crate::domain_computation) fn read_binding(
    role: &str,
    access: WorthQueryOperationGraphAccess,
) -> String {
    match access {
        WorthQueryOperationGraphAccess::Observe => format!("{role}:observe"),
        WorthQueryOperationGraphAccess::Project => format!("{role}:project"),
    }
}

pub(in crate::domain_computation) fn effect_families(
    contract: &WorthQueryOperationEffectContract,
) -> Vec<String> {
    match contract {
        WorthQueryOperationEffectContract::NotRequired => Vec::new(),
        WorthQueryOperationEffectContract::Declared { effect_families } => effect_families
            .iter()
            .map(|effect| effect.as_str().to_owned())
            .collect(),
    }
}

pub(in crate::domain_computation) fn invariant_slots(
    contract: &WorthQueryOperationInvariantContract,
) -> Vec<String> {
    match contract {
        WorthQueryOperationInvariantContract::NotRequired => Vec::new(),
        WorthQueryOperationInvariantContract::Declared { invariant_slots } => {
            invariant_slots.clone()
        }
    }
}

impl WorthQueryProviderPlanDeclaredClosure {
    #[doc(hidden)]
    pub fn effect_for_test(&self) -> &Vec<String> {
        &self.effect
    }
}
