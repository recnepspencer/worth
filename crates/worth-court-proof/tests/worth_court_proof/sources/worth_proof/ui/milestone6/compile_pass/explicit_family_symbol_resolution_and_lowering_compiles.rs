use worth_proof::{
    lower_deterministic_family_pair, resolve_family_symbol, AuthoritativeFamilyMember,
    CompositionFamilySymbol, FamilyLifecycleAction, Pair,
};

fn family_action_key(
    action: &FamilyLifecycleAction<u8, u16, &'static str>,
) -> (u8, Option<u8>, Option<u16>) {
    match action {
        FamilyLifecycleAction::Retire { target } => (0, None, Some(*target.value())),
        FamilyLifecycleAction::Rewrite { target, .. } => (1, None, Some(*target.value())),
        FamilyLifecycleAction::Supersede { target, .. } => (2, None, Some(*target.value())),
        FamilyLifecycleAction::Create { symbol, .. } => (3, Some(*symbol.value()), None),
    }
}

fn explicit_family_symbol_resolution_and_lowering_compiles() {
    let symbol = CompositionFamilySymbol::new(2_u8);
    let authoritative = AuthoritativeFamilyMember::new(11_u16);
    let resolved = resolve_family_symbol(symbol.clone(), authoritative.clone());

    let lowered = lower_deterministic_family_pair(
        Pair::new(
            FamilyLifecycleAction::Create {
                symbol,
                payload: "create",
            },
            FamilyLifecycleAction::Supersede {
                target: resolved.into_authoritative(),
                replacement: CompositionFamilySymbol::new(3_u8),
                payload: "replace",
            },
        ),
        family_action_key,
    );

    let _ = lowered.actions();
    let _ = lowered.proof();
}

fn main() {}
