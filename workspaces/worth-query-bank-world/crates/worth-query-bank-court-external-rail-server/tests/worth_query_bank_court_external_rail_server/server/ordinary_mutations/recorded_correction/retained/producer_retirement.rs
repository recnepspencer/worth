//! Real full-Bank installation owners cross only the quiescent graceful boundary.
use super::{close_directory, fixture};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement,
    primary_graph::{WorthQueryHomeRetirementDenial, WorthQueryHomeRetirementOwner},
};

fn producer_identities(home: &fixture::BankHome) -> Vec<String> {
    let identities = home
        .runtime()
        .runtime()
        .installed_producers_for_test()
        .installed_identities_for_test()
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert!(
        !identities.is_empty(),
        "full Bank installs genuine producers"
    );
    identities
}

fn refused_home(home: fixture::BankHome) -> fixture::BankHome {
    let original = home.runtime().runtime() as *const _ as usize;
    let refused = with_bootstrap_advancement(fixture::home_policy(), |phase| {
        home.close(&phase.execution_request())
    })
    .unwrap()
    .err()
    .expect("retained executable/source custody refuses close");
    assert!(matches!(
        refused.query_denial(),
        Some(WorthQueryHomeRetirementDenial::Busy(
            WorthQueryHomeRetirementOwner::InstalledProducers
        ))
    ));
    let resumed = refused.resume().unwrap();
    assert_eq!(resumed.runtime().runtime() as *const _ as usize, original);
    resumed
}

#[test]
fn full_bank_nonempty_producers_retire_reclaim_and_reopen_unchanged() {
    let world = fixture::DirectoryWorld::new();
    let home = world.open();
    let identities = producer_identities(&home);
    let revision = *home.runtime().installed_program().revision();
    let owner = home
        .runtime()
        .runtime()
        .product_runtime()
        .root_product_branch()
        .occurrence_for_test()
        .owner_identity()
        .durable()
        .expect("Directory World owner");
    {
        let principals = world.authenticate(&home);
        assert_eq!(
            fixture::DirectoryWorld::balances(&home, &principals),
            (100, 0)
        );
    }
    close_directory(home);
    let reopened = world.open();
    assert_eq!(producer_identities(&reopened), identities);
    assert_eq!(*reopened.runtime().installed_program().revision(), revision);
    assert_eq!(
        reopened
            .runtime()
            .runtime()
            .product_runtime()
            .root_product_branch()
            .occurrence_for_test()
            .owner_identity()
            .durable(),
        Some(owner)
    );
    {
        let principals = world.authenticate(&reopened);
        assert_eq!(
            fixture::DirectoryWorld::balances(&reopened, &principals),
            (100, 0)
        );
    }
    close_directory(reopened);
}

#[test]
fn full_bank_executable_aliases_and_source_custody_preserve_the_refused_home() {
    let world = fixture::DirectoryWorld::new();
    let mut home = world.open();
    let identities = producer_identities(&home);
    for kind in 0..4 {
        let registry = home.runtime().runtime().installed_producers_for_test();
        let alias = match kind {
            0 => registry.retain_entry_for_test(),
            1 => registry.retain_weak_entry_for_test(),
            2 => registry.retain_executor_for_test(),
            _ => registry.retain_weak_executor_for_test(),
        }
        .expect("retain an actual installed Bank owner");
        home = refused_home(home);
        assert_eq!(producer_identities(&home), identities);
        drop(alias);
    }
    let occurrence = home
        .runtime()
        .runtime()
        .product_runtime()
        .root_product_branch()
        .occurrence_for_test();
    let preparation = home
        .runtime()
        .runtime()
        .runtime_for_test()
        .output_demands_for_test()
        .begin_source_preparation_for_test(occurrence);
    home = refused_home(home);
    assert_eq!(producer_identities(&home), identities);
    drop(preparation);
    close_directory(home);
    let reopened = world.open();
    assert_eq!(producer_identities(&reopened), identities);
    {
        let principals = world.authenticate(&reopened);
        assert_eq!(
            fixture::DirectoryWorld::balances(&reopened, &principals),
            (100, 0)
        );
    }
    close_directory(reopened);
}
