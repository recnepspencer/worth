use worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::declared_closure::application::bind_application_candidate_effect_closure;
use worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::declared_closure::bind_direct_role_closure;
use worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::declared_closure::aftermath_posture::reversal_posture;
use worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::declared_closure::application::WorthQueryApplicationEffectPosture;
use worth_query_execution::court_doors_for_test::domain_computation::provider_session::protocol::declared_closure::WorthQueryProviderPlanDeclaredClosure;
use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture as fixture;

#[test]
fn read_only_roles_receive_no_effect_authority() {
    let effects = vec!["mutation".to_owned()];
    let invariants = vec!["closed-loop".to_owned()];
    let mut read_only = {
        let __read = vec!["read-only:observe".to_owned()];
        let mut __base = WorthQueryProviderPlanDeclaredClosure::default();
        *__base.read_mut_for_test() = __read;
        __base
    };
    bind_direct_role_closure(&mut read_only, &effects, &invariants);
    assert!((*read_only.effect_for_test()).is_empty());

    let mut touched = {
        let __read = vec!["owner:observe".to_owned()];
        let __touch = vec!["owner".to_owned()];
        let mut __base = WorthQueryProviderPlanDeclaredClosure::default();
        *__base.read_mut_for_test() = __read;
        *__base.touch_mut_for_test() = __touch;
        __base
    };
    bind_direct_role_closure(&mut touched, &effects, &invariants);
    assert_eq!((*touched.effect_for_test()), effects);
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
    assert_eq!((*closure.effect_for_test()), ["payment", "mutation"]);
    assert_eq!((*closure.invariant_for_test()), invariants);

    bind_application_candidate_effect_closure(
        &mut closure,
        WorthQueryApplicationEffectPosture::Platform,
        true,
        &effects,
        &invariants,
    );
    assert_eq!((*closure.effect_for_test()), ["mutation"]);
    assert!((*closure.invariant_for_test()).is_empty());

    bind_application_candidate_effect_closure(
        &mut closure,
        WorthQueryApplicationEffectPosture::Application,
        true,
        &effects,
        &invariants,
    );
    assert_eq!((*closure.effect_for_test()), effects);
    assert_eq!((*closure.invariant_for_test()), invariants);
}

#[test]
fn reversal_posture_retains_the_exact_declared_recovery_contract() {
    let inverse = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture::freeze_balance();
    assert_eq!(
        reversal_posture(Some(&inverse)),
        "exact-inverse:unfreeze:inverse-v2:exact-prior-truth"
    );

    let compensation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::application_aftermath::aftermath_schema_fixture::charge();
    assert_eq!(
        reversal_posture(Some(&compensation)),
        "compensation:undo-charge:invariant-restored:balanced-ledger"
    );
}
