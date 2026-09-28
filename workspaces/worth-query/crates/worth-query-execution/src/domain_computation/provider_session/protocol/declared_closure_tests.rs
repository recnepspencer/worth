use super::declared_closure::{
    bind_application_candidate_effect_closure, bind_direct_role_closure, reversal_posture,
    WorthQueryApplicationEffectPosture, WorthQueryProviderPlanDeclaredClosure,
};
use crate::domain_computation::application_aftermath::aftermath_schema_fixture as fixture;

#[test]
fn read_only_roles_receive_no_effect_authority() {
    let effects = vec!["mutation".to_owned()];
    let invariants = vec!["closed-loop".to_owned()];
    let mut read_only = WorthQueryProviderPlanDeclaredClosure {
        read: vec!["read-only:observe".to_owned()],
        ..WorthQueryProviderPlanDeclaredClosure::default()
    };
    bind_direct_role_closure(&mut read_only, &effects, &invariants);
    assert!(read_only.effect.is_empty());

    let mut touched = WorthQueryProviderPlanDeclaredClosure {
        read: vec!["owner:observe".to_owned()],
        touch: vec!["owner".to_owned()],
        ..WorthQueryProviderPlanDeclaredClosure::default()
    };
    bind_direct_role_closure(&mut touched, &effects, &invariants);
    assert_eq!(touched.effect, effects);
}

#[test]
fn mixed_application_candidate_retains_domain_and_platform_effect_closure() {
    let effects = vec!["payment".to_owned()];
    let invariants = vec!["balance".to_owned()];
    let mut closure = WorthQueryProviderPlanDeclaredClosure::default();
    bind_application_candidate_effect_closure(
        &mut closure,
        WorthQueryApplicationEffectPosture::ApplicationWithPlatform,
        true,
        &effects,
        &invariants,
    );
    assert_eq!(closure.effect, ["payment", "mutation"]);
    assert_eq!(closure.invariant, invariants);

    bind_application_candidate_effect_closure(
        &mut closure,
        WorthQueryApplicationEffectPosture::Platform,
        true,
        &effects,
        &invariants,
    );
    assert_eq!(closure.effect, ["mutation"]);
    assert!(closure.invariant.is_empty());

    bind_application_candidate_effect_closure(
        &mut closure,
        WorthQueryApplicationEffectPosture::Application,
        true,
        &effects,
        &invariants,
    );
    assert_eq!(closure.effect, effects);
    assert_eq!(closure.invariant, invariants);
}

#[test]
fn reversal_posture_retains_the_exact_declared_recovery_contract() {
    let inverse = fixture::freeze_balance();
    assert_eq!(
        reversal_posture(Some(&inverse)),
        "exact-inverse:unfreeze:inverse-v2:exact-prior-truth"
    );

    let compensation = fixture::charge();
    assert_eq!(
        reversal_posture(Some(&compensation)),
        "compensation:undo-charge:invariant-restored:balanced-ledger"
    );
}
