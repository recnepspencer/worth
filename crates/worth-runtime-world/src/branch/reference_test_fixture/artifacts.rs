use std::sync::Arc;

use crate::branch::reference_cell::ProductBranchHeadProtection;
use crate::branch::ProductBranchReferenceSnapshot;
use crate::history::{
    CompositeHistoryCatalog, CompositeRuntimeWorldCommit, RuntimeWorldHistoryCatalogContract,
};
use crate::identity::ProductBranchReferenceGeneration;
use crate::publication::CompositeOwnerExecutionResults;

use super::RealReferenceFixture;

pub(crate) fn history_catalog(
    owner: crate::identity::RuntimeWorldOwnerIdentity,
) -> CompositeHistoryCatalog {
    let budgets = crate::budget::RuntimeWorldBudgets::install(
        crate::budget::RuntimeWorldBudgetInstallation {
            branches: crate::budget::RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 1,
            },
            history: crate::budget::RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 16,
                history_metadata_bytes: 4096,
            },
            observations: crate::budget::RuntimeWorldObservationBudgetInstallation {
                active_observations: 8,
            },
            publication: crate::budget::RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 8,
            },
            recovery: crate::budget::RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 1,
                retained_partial_metadata_bytes: 1,
            },
            retention: crate::budget::RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 8,
                in_flight_pin_acquisition_reservations: 8,
            },
            custody: crate::budget::RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 1,
            },
        },
    )
    .expect("positive history test budgets");
    CompositeHistoryCatalog::new(
        owner,
        RuntimeWorldHistoryCatalogContract::installed(
            budgets.retained_composite_commits(),
            budgets.history_metadata_bytes(),
        ),
    )
}

pub(crate) fn root_commit(fixture: &mut RealReferenceFixture) -> CompositeRuntimeWorldCommit {
    CompositeRuntimeWorldCommit::from_root_bootstrap(
        fixture
            .identities
            .issuer_mut()
            .composite_commit()
            .expect("root commit identity"),
        fixture.basis.clone(),
        fixture
            .identities
            .issuer_mut()
            .bootstrap_attempt()
            .expect("root bootstrap identity"),
        None,
    )
    .expect("explicit root commit from admitted basis")
}

pub(crate) fn ordinary_commit(
    fixture: &mut RealReferenceFixture,
    predecessor: &CompositeRuntimeWorldCommit,
) -> CompositeRuntimeWorldCommit {
    CompositeRuntimeWorldCommit::from_ordinary_publication(
        fixture
            .identities
            .issuer_mut()
            .composite_commit()
            .expect("ordinary commit identity"),
        predecessor,
        fixture.basis.clone(),
        fixture
            .identities
            .issuer_mut()
            .publication_attempt()
            .expect("publication attempt identity"),
        &CompositeOwnerExecutionResults::retained(),
        None,
    )
    .expect("explicit ordinary commit from same admitted basis")
}

pub(crate) fn installed_root() -> (
    RealReferenceFixture,
    CompositeHistoryCatalog,
    Arc<CompositeRuntimeWorldCommit>,
) {
    let mut fixture = super::real_fixture(16, 16);
    let root = Arc::new(root_commit(&mut fixture));
    let catalog = history_catalog(fixture.owner_identity);
    catalog
        .append(
            Arc::clone(&root),
            fixture
                .owner
                .issue_publication(root.basis())
                .unwrap()
                .fork_history(root.basis())
                .unwrap(),
        )
        .expect("root install");
    (fixture, catalog, root)
}

pub(crate) fn install_ordinary(
    fixture: &mut RealReferenceFixture,
    catalog: &CompositeHistoryCatalog,
    predecessor: &CompositeRuntimeWorldCommit,
) -> Arc<CompositeRuntimeWorldCommit> {
    let commit = Arc::new(ordinary_commit(fixture, predecessor));
    catalog
        .append(
            Arc::clone(&commit),
            fixture
                .owner
                .issue_publication(commit.basis())
                .unwrap()
                .fork_history(commit.basis())
                .unwrap(),
        )
        .expect("ordinary commit install");
    commit
}

pub(crate) fn initial_snapshot(
    fixture: &mut RealReferenceFixture,
    commit: Arc<CompositeRuntimeWorldCommit>,
) -> ProductBranchReferenceSnapshot {
    initial_snapshot_named(fixture, commit, "root")
}

/// An owner-issued identity is keyed by the normalized branch name, so a
/// snapshot for a genuinely different product branch is named, not counted.
pub(crate) fn initial_snapshot_named(
    fixture: &mut RealReferenceFixture,
    commit: Arc<CompositeRuntimeWorldCommit>,
    name: &str,
) -> ProductBranchReferenceSnapshot {
    let branch = crate::identity::ProductBranchIdentity::issued(
        fixture.owner_identity,
        crate::branch::ProductBranchName::try_new(name).expect("valid product branch name"),
    );
    let incarnation = fixture
        .identities
        .issuer_mut()
        .branch_incarnation()
        .expect("branch incarnation identity");
    ProductBranchReferenceSnapshot::owner_issued(
        fixture.owner_identity,
        branch,
        incarnation,
        ProductBranchReferenceGeneration::initial(),
        commit,
    )
    .expect("coherent initial reference snapshot")
}

pub(crate) fn successor_snapshot(
    current: &ProductBranchReferenceSnapshot,
    commit: Arc<CompositeRuntimeWorldCommit>,
) -> ProductBranchReferenceSnapshot {
    ProductBranchReferenceSnapshot::owner_issued(
        current.owner(),
        current.branch().clone(),
        current.lifecycle(),
        current.generation().advance().expect("generation advance"),
        commit,
    )
    .expect("coherent successor reference snapshot")
}

pub(crate) fn product_head_protection(
    fixture: &RealReferenceFixture,
    catalog: &CompositeHistoryCatalog,
    snapshot: ProductBranchReferenceSnapshot,
) -> ProductBranchHeadProtection {
    let publication = fixture
        .owner
        .issue_publication(snapshot.commit().basis())
        .expect("real publication retention pair");
    let transfer = publication
        .into_product_head_transfer(snapshot.commit().basis())
        .expect("publication retention binds the exact successor basis");
    let history = catalog
        .protect_product_head(snapshot.commit())
        .expect("installed successor product-head history protection");
    ProductBranchHeadProtection::owner_issued(snapshot, transfer, history)
        .expect("component and history head protection correspond exactly")
}

pub(crate) fn bootstrap_product_head_protection(
    fixture: &RealReferenceFixture,
    catalog: &CompositeHistoryCatalog,
    snapshot: ProductBranchReferenceSnapshot,
) -> ProductBranchHeadProtection {
    let product_head = fixture
        .owner
        .issue_product_head(snapshot.commit().basis())
        .expect("real direct product-head retention pair");
    let history = catalog
        .protect_product_head(snapshot.commit())
        .expect("installed root product-head history protection");
    ProductBranchHeadProtection::bootstrap_issued(snapshot, product_head, history)
        .expect("direct product-head and history protections correspond")
}
