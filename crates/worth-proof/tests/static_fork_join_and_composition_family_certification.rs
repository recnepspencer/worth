#[allow(unused_imports)]
use crate::shared_support as support;

use support::compile_fail::run_compile_fail_bundle;
use support::compile_pass::run_compile_pass_bundle;
use support::composition_family;
use worth_proof::{
    lower_deterministic_family_pair, resolve_family_symbol, AuthoritativeFamilyMember,
    CompositionFamilySymbol, FamilyLifecycleAction,
};

#[test]
fn static_fork_join_and_composition_family_certification() {
    let compile_fail_bundle = composition_family::compile_fail_bundle();
    let compile_pass_bundle = composition_family::compile_pass_bundle();
    let transition_digest = composition_family::transition_digest();
    let composition_digest = composition_family::composition_digest();
    let proof_shape_digest = composition_family::proof_shape_digest();
    let failure_digest = composition_family::failure_digest();
    let codegen_honesty_report = composition_family::codegen_honesty_report();
    let residual_debt_report = composition_family::residual_debt_report();

    run_compile_fail_bundle(&compile_fail_bundle);
    run_compile_pass_bundle(&compile_pass_bundle);

    assert_eq!(
        compile_fail_bundle.suite(),
        "static_fork_join_and_composition_family"
    );
    assert_eq!(
        compile_fail_bundle.families(),
        vec![
            "wrong_join_shape",
            "wrong_fork_shape",
            "lowered_ready_join_boundary",
            "family_identity_boundary",
        ]
    );
    assert_eq!(
        compile_pass_bundle.suite(),
        "static_fork_join_and_composition_family"
    );
    assert_eq!(
        compile_pass_bundle.families(),
        vec![
            "explicit_fixed_arity_fork_join",
            "checked_multi_input_ordering_and_ready_join",
            "explicit_family_symbol_resolution_and_lowering",
        ]
    );

    assert_eq!(
        transition_digest.suite(),
        "static_fork_join_and_composition_family"
    );
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("compose_join_transition_outcome")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("compose_join_success_transition")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("compose_join_ready_recipe_pair")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("family_lifecycle::create")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("family_lifecycle::rewrite")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("family_lifecycle::supersede")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("family_lifecycle::retire")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("deterministic_family_lowering::create_retire_converges")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("deterministic_family_lowering::rewrite_supersede_converges")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("non_success_short_circuit::left_denial")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry
            .contains("family_identity_boundary::symbolic_and_authoritative_are_distinct")));
    assert!(transition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("non_success_short_circuit::right_denial")));

    assert_eq!(
        composition_digest.suite(),
        "static_fork_join_and_composition_family"
    );
    assert_eq!(
        proof_shape_digest.suite(),
        "static_fork_join_and_composition_family"
    );
    assert_eq!(proof_shape_digest.entries(), composition_digest.entries());
    assert!(composition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("ForkOutputs2")));
    assert!(composition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("JoinInputs2")));
    assert!(composition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("CompositionFamilySymbol")));
    assert!(composition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("AuthoritativeFamilyMember")));
    assert!(composition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("LoweredFamilyProgram2")));
    assert!(composition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("resolve_family_symbol")));
    assert!(composition_digest
        .entries()
        .iter()
        .any(|entry| entry.contains("lower_deterministic_family_pair")));

    assert_eq!(
        failure_digest.suite(),
        "static_fork_join_and_composition_family"
    );
    assert_eq!(
        failure_digest.entries(),
        [
            "wrong_join_shape::tests/ui/milestone6/compile_fail/raw_vec_cannot_satisfy_join_inputs.rs",
            "wrong_fork_shape::tests/ui/milestone6/compile_fail/raw_tuple_cannot_satisfy_fork_outputs.rs",
            "lowered_ready_join_boundary::tests/ui/milestone6/compile_fail/lowered_recipe_cannot_satisfy_ready_join.rs",
            "family_identity_boundary::tests/ui/milestone6/compile_fail/symbolic_family_reference_cannot_satisfy_authoritative_api.rs",
            "short_circuit::left_denial_skips_right_lane",
            "short_circuit::right_denial_skips_next_step",
            "equivalence_lane::create_retire_family_lowering_converges",
            "equivalence_lane::rewrite_supersede_family_lowering_converges",
            "identity_divergence::symbolic_family_reference_is_not_authoritative_identity",
        ]
    );

    assert_eq!(
        codegen_honesty_report.suite(),
        "static_fork_join_and_composition_family"
    );
    assert_eq!(
        codegen_honesty_report.verified_scope(),
        "size_layout_and_drop_only"
    );
    assert!(codegen_honesty_report
        .checks()
        .iter()
        .all(|check| check.matches()));
    assert!(!codegen_honesty_report.hidden_dynamic_lookup());
    assert!(!codegen_honesty_report.hidden_virtual_dispatch());
    assert!(!codegen_honesty_report.mandatory_allocation_introduced());

    assert_eq!(
        residual_debt_report.suite(),
        "static_fork_join_and_composition_family"
    );
    assert_eq!(residual_debt_report.items().len(), 1);
    assert_eq!(
        residual_debt_report.items()[0].category(),
        "representative_scope"
    );

    assert_ne!(
        std::any::type_name::<CompositionFamilySymbol<u8>>(),
        std::any::type_name::<AuthoritativeFamilyMember<u8>>()
    );

    let create = FamilyLifecycleAction::Create {
        symbol: CompositionFamilySymbol::new(9_u8),
        payload: "create",
    };
    let retire = FamilyLifecycleAction::Retire {
        target: AuthoritativeFamilyMember::new(4_u16),
    };
    let lowered_left = lower_deterministic_family_pair(
        worth_proof::Pair::new(create.clone(), retire.clone()),
        family_action_key,
    );
    let lowered_right =
        lower_deterministic_family_pair(worth_proof::Pair::new(retire, create), family_action_key);

    assert_eq!(lowered_left.actions(), lowered_right.actions());

    let rewrite = FamilyLifecycleAction::Rewrite {
        target: AuthoritativeFamilyMember::new(3_u16),
        payload: "rewrite",
    };
    let supersede = FamilyLifecycleAction::Supersede {
        target: AuthoritativeFamilyMember::new(11_u16),
        replacement: CompositionFamilySymbol::new(2_u8),
        payload: "replace",
    };
    let lowered_rewrite_supersede = lower_deterministic_family_pair(
        worth_proof::Pair::new(supersede.clone(), rewrite.clone()),
        family_action_key,
    );
    let lowered_supersede_rewrite = lower_deterministic_family_pair(
        worth_proof::Pair::new(rewrite, supersede),
        family_action_key,
    );

    assert_eq!(
        lowered_rewrite_supersede.actions(),
        lowered_supersede_rewrite.actions()
    );

    let resolved = resolve_family_symbol(
        CompositionFamilySymbol::new(3_u8),
        AuthoritativeFamilyMember::new(7_u16),
    );
    assert_eq!(resolved.symbol().value(), &3_u8);
    assert_eq!(resolved.authoritative().value(), &7_u16);
}

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
