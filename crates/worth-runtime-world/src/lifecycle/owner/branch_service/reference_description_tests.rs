//! A transported name never selects a retired or replacement occurrence.
use super::tests::{create_reused_branch, reuse_intent, setup};
use crate::branch::{
    ProductBranchReferenceDescription, ProductBranchReferenceReadmissionDenial as Denial,
};
use crate::lifecycle::{RuntimeWorldBranchService, RuntimeWorldObservationService};

#[test]
fn stored_reference_description_refuses_retirement_and_same_name_recreation() {
    let (_fixture, owner, source) = setup(3);
    let branch = create_reused_branch(&owner, &source, reuse_intent("stored-candidate"));
    let description = ProductBranchReferenceDescription::from_observation(&branch);
    let retirement = owner.retire_product_branch(&branch).unwrap();
    assert!(retirement.owner_retirement_work().is_empty());
    drop(branch);
    assert!(matches!(
        owner.readmit_product_branch_reference(&description),
        Err(Denial::Branch(
            crate::branch::RuntimeWorldBranchAdmissionDenial::RetiredBranch
        ))
    ));
    let replacement = create_reused_branch(&owner, &source, reuse_intent("stored-candidate"));
    assert_ne!(
        description.lifecycle_ordinal,
        replacement.lifecycle_incarnation().ordinal()
    );
    assert!(matches!(
        owner.readmit_product_branch_reference(&description),
        Err(Denial::IncarnationChanged)
    ));
    let fresh = ProductBranchReferenceDescription::from_observation(&replacement);
    assert_eq!(
        owner.readmit_product_branch_reference(&fresh).unwrap(),
        replacement
    );
    assert_eq!(owner.state.operation.active(), 0);
}
