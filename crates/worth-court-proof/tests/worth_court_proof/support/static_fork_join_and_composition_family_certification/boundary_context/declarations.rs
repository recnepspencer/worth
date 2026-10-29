#[allow(unused_imports)]
pub(crate) use {crate::shared_support as support};

pub(crate) use {crate::shared_support::compile_fail::run_compile_fail_bundle};
pub(crate) use {crate::shared_support::compile_pass::run_compile_pass_bundle};
pub(crate) use {crate::shared_support::composition_family};
pub(crate) use {worth_proof::lower_deterministic_family_pair, worth_proof::resolve_family_symbol, worth_proof::AuthoritativeFamilyMember, worth_proof::CompositionFamilySymbol, worth_proof::FamilyLifecycleAction};

pub(crate) fn family_action_key(
    action: &FamilyLifecycleAction<u8, u16, &'static str>,
) -> (u8, Option<u8>, Option<u16>) {
    match action {
        worth_proof::FamilyLifecycleAction::Retire { target } => (0, None, Some(*target.value())),
        worth_proof::FamilyLifecycleAction::Rewrite { target, .. } => (1, None, Some(*target.value())),
        worth_proof::FamilyLifecycleAction::Supersede { target, .. } => (2, None, Some(*target.value())),
        worth_proof::FamilyLifecycleAction::Create { symbol, .. } => (3, Some(*symbol.value()), None),
    }
}