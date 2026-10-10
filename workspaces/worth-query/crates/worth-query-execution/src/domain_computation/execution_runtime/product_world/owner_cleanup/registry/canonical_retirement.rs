//! Retired descriptions retain their declared branch and occurrence order.
use super::*;
use worth_runtime_world::facade::*;

#[test]
fn equal_contents_retire_in_the_same_declared_branch_sequence() {
    let world =
        crate::domain_computation::primary_graph::tests::fixture::installed_authorization_world(
            true,
        );
    let runtime = world.application.product_runtime();
    let root = runtime
        .admit_product_branch(runtime.default_branch())
        .unwrap();
    let child = |name| {
        let intent = ProductBranchCreationIntent::from_source(
            name,
            ProductBranchCreationPlans::new(
                RelationalBranchCreationPlan::ReuseExact,
                SignalBranchCreationPlan::ReuseExact,
            ),
        )
        .unwrap();
        let RuntimeWorldBranchCreationOutcome::Performed(child) = runtime
            .create_product_branch(
                &root,
                None,
                intent,
                &RuntimeWorldCancellationSource::new().token(),
            )
            .unwrap()
        else {
            panic!("genuine child occurrence");
        };
        let occurrence = WorthQueryRetiredProductOccurrence::new(
            child.branch_identity().clone(),
            child.lifecycle_incarnation(),
        );
        let observed = runtime
            .admit_product_occurrence(child.lifecycle_incarnation())
            .unwrap();
        let report = runtime.retire_product_branch(&observed).unwrap();
        (occurrence, report)
    };
    let (zulu, _zulu_report) = child("zulu");
    let (alpha, _alpha_report) = child("alpha");
    let sequence = |registry: &WorthQueryProductBranchOwnerCleanupRegistry| {
        registry
            .pending_application_retired_product_occurrences()
            .into_iter()
            .map(|occurrence| (occurrence.branch().clone(), occurrence.incarnation()))
            .collect::<Vec<_>>()
    };
    for _ in 0..16 {
        let install = |entries: [(u64, WorthQueryRetiredProductOccurrence); 2]| {
            let registry = WorthQueryProductBranchOwnerCleanupRegistry::new(2);
            for (identity, occurrence) in entries {
                // This read asks only for descriptions, including while a
                // retry has taken the record; it grants no cleanup authority.
                lock(&registry.state).entries.insert(
                    identity,
                    Arc::new(CleanupEntry {
                        record: Mutex::new(None),
                        scope: CleanupScope::ApplicationRetirement(occurrence),
                    }),
                );
            }
            registry
        };
        let left = install([(1, zulu.clone()), (2, alpha.clone())]);
        let right = install([(2, alpha.clone()), (1, zulu.clone())]);
        let a = sequence(&left);
        let b = sequence(&right);
        assert_eq!(a, b);
        assert_eq!(
            a,
            [
                (alpha.branch().clone(), alpha.incarnation()),
                (zulu.branch().clone(), zulu.incarnation())
            ]
        );
    }
}
