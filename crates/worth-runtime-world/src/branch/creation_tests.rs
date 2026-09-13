use super::{
    ProductBranchCreationIntent, ProductBranchCreationPlans, ProductBranchNameDenial,
    RelationalBranchCreationPlan, SignalBranchCreationPlan,
};

use worth_relational::facade::history::BranchId;

fn relational_fork() -> RelationalBranchCreationPlan {
    RelationalBranchCreationPlan::ForkExact {
        target: BranchId("relational-child".to_owned()),
    }
}

fn signal_fork() -> SignalBranchCreationPlan {
    SignalBranchCreationPlan::ForkExact {
        target: worth_signal::facade::branch::validate_signal_branch_name("signal-child")
            .expect("Signal branch name validates"),
    }
}

#[test]
fn creation_plans_expose_explicit_reuse_and_owner_effect_meaning() {
    assert!(RelationalBranchCreationPlan::ReuseExact.is_reuse_exact());
    assert!(!RelationalBranchCreationPlan::ReuseExact.requires_owner_effect());
    assert!(relational_fork().requires_owner_effect());
    assert!(SignalBranchCreationPlan::ReuseExact.is_reuse_exact());
    assert!(signal_fork().requires_owner_effect());

    let reuse = ProductBranchCreationPlans::new(
        RelationalBranchCreationPlan::ReuseExact,
        SignalBranchCreationPlan::ReuseExact,
    );
    assert!(reuse.is_exact_reuse());
    assert!(!reuse.requires_relational_owner_effect());
    assert!(!reuse.requires_signal_owner_effect());

    let mixed =
        ProductBranchCreationPlans::new(relational_fork(), SignalBranchCreationPlan::ReuseExact);
    assert!(!mixed.is_exact_reuse());
    assert!(mixed.requires_relational_owner_effect());
    assert!(!mixed.requires_signal_owner_effect());
}

#[test]
fn each_fork_plan_carries_its_own_owner_issued_destination() {
    let plans = ProductBranchCreationPlans::new(relational_fork(), signal_fork());
    assert_eq!(
        plans.relational().fork_target(),
        Some(&BranchId("relational-child".to_owned()))
    );
    assert_eq!(
        plans.signal().fork_target(),
        Some(
            &worth_signal::facade::branch::validate_signal_branch_name("signal-child")
                .expect("Signal branch name validates")
        )
    );
    assert!(ProductBranchCreationPlans::new(
        RelationalBranchCreationPlan::ReuseExact,
        SignalBranchCreationPlan::ReuseExact,
    )
    .relational()
    .fork_target()
    .is_none());
}

#[test]
fn branch_creation_name_is_validated_before_identity_issuance() {
    assert_eq!(
        ProductBranchCreationIntent::named("   ").expect_err("blank names are not meaning"),
        ProductBranchNameDenial::Empty
    );
    assert!(ProductBranchCreationIntent::named("child").is_ok());
    assert!(ProductBranchCreationIntent::named("x".repeat(257)).is_err());
}

#[test]
fn a_bootstrap_intent_carries_no_creation_plans() {
    let bootstrap = ProductBranchCreationIntent::named("root").expect("valid root name");
    assert!(bootstrap.plans().is_none());

    let from_source = ProductBranchCreationIntent::from_source(
        "child",
        ProductBranchCreationPlans::new(relational_fork(), signal_fork()),
    )
    .expect("valid child name");
    assert!(from_source.plans().is_some());
}
