use super::super::{CompositeHistoryCatalog, CompositeHistoryCatalogDenial};
use super::allocation_oracle::AllocationOracle;
use super::fixtures::{history_contract, linear_history};
use crate::branch::{ProductBranchName, ProductBranchReferenceSnapshot};
use crate::history::{CompositeCommitProvenance, CompositeHistoryReclamationRequest};
use crate::identity::{ProductBranchIdentity, ProductBranchReferenceGeneration};

#[test]
fn publication_envelope_is_charged_before_effects_and_released_with_its_entry() {
    let (mut owner, commits) = linear_history(2);
    let root = &commits[0];
    let child = &commits[1];
    let CompositeCommitProvenance::Publication(attempt) = child.provenance() else {
        panic!("ordinary child")
    };
    let expected = ProductBranchReferenceSnapshot::owner_issued(
        owner.authority.owner_identity(),
        ProductBranchIdentity::issued(
            owner.authority.owner_identity(),
            ProductBranchName::try_new("root").unwrap(),
        ),
        owner.authority.issuer_mut().branch_incarnation().unwrap(),
        ProductBranchReferenceGeneration::initial(),
        root.clone(),
    )
    .unwrap();
    let root_charge = AllocationOracle::installed_resident(root);
    let publication_charge = AllocationOracle::publication_resident(child, "root");
    let maximum = root_charge + AllocationOracle::reservation_resident(child) + publication_charge;
    let denied = CompositeHistoryCatalog::new(
        owner.authority.owner_identity(),
        history_contract(2, (maximum - 1) as u64),
    );
    denied
        .append(root.clone(), owner.history_pins(root))
        .unwrap();
    let before = denied.metadata_ledger();
    assert!(matches!(
        denied.reserve_publication_capacity(
            child.identity().clone(),
            attempt.clone(),
            expected.clone()
        ),
        Err(CompositeHistoryCatalogDenial::MetadataCapacityExhausted { .. })
    ));
    assert_eq!(denied.metadata_ledger(), before);
    assert_eq!(denied.reserved_len(), 0);

    let catalog = CompositeHistoryCatalog::new(
        owner.authority.owner_identity(),
        history_contract(2, maximum as u64),
    );
    catalog
        .append(root.clone(), owner.history_pins(root))
        .unwrap();
    let mut capacity = catalog
        .reserve_publication_capacity(child.identity().clone(), attempt.clone(), expected)
        .unwrap();
    assert_eq!(catalog.metadata_ledger().total_occupancy(), maximum);
    assert_eq!(
        catalog.metadata_ledger().promised_installation(),
        publication_charge
    );
    assert!(catalog
        .claim_performed_publication(child.identity())
        .unwrap()
        .is_none());
    let (head, delivery) = capacity
        .try_install_publication(child.clone(), owner.history_pins(child))
        .unwrap();
    assert_eq!(
        catalog.metadata_ledger().installed_resident(),
        root_charge + publication_charge
    );
    assert_eq!(catalog.metadata_ledger().reservation_resident(), 0);
    assert!(
        catalog
            .claim_performed_publication(child.identity())
            .unwrap()
            .is_none(),
        "materializing a commit does not attest a product movement"
    );
    drop(capacity);
    drop(head);
    let request = || {
        CompositeHistoryReclamationRequest::new(
            owner.authority.owner_identity(),
            vec![child.identity().clone()],
            1,
        )
    };
    assert_eq!(
        catalog
            .reclaim_batch(request())
            .unwrap()
            .skipped_protected(),
        1
    );
    drop(delivery);
    assert_eq!(
        catalog
            .reclaim_batch(request())
            .unwrap()
            .reclaimed_commits(),
        &[child.identity().clone()]
    );
    assert_eq!(catalog.metadata_ledger().total_occupancy(), root_charge);
}
