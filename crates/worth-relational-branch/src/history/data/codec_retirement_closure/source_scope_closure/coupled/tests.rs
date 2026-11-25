//! Pure exact-owner assertions, borrowing the existing codec unit host's context.
use super::*;
use crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning;
use crate::mvcc::publication::application_publication::producing_implementation::RequiredOutputProducingImplementation;
#[doc(hidden)]
pub fn assert_exact_owner_refusals_for_test(
    receipt: crate::history::data::RelationalCommitReceipt,
    basis: &crate::branch::basis::RelationalBranchBasisDescriptor,
    source: EntityId,
    context: &mut MapKernelContext<'_, '_>,
) {
    let meaning = RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint([1; 16], [2; 32])
            .unwrap(),
        family: "owner-selection-data".into(),
        source_scope: source,
        source_partition: worth_foundational::CanonicalDigestId::new([3; 32]),
        source_epoch: worth_foundational::CanonicalDigestId::new([4; 32]),
    };
    // No installed issuer is fabricated. These are comparison inputs only;
    // complete publication/header/readmission still have their other gates.
    let value = serde_json::json!({
        "commit": receipt, "basis": basis,
        "producer": "owner-selection-data", "source_scope_owned": true,
        "dependency_identity": worth_foundational::CanonicalDigestId::new([5;32]),
        "inputs": [], "outputs": [], "relations": [], "meaning": meaning, "publication": "Birth",
        "birth": receipt.commit_id, "birth_producer": "owner-selection-data",
        "birth_dependency": worth_foundational::CanonicalDigestId::new([6;32]), "members": []
    });
    let expected: CanonicalRequiredOutputOrigin = serde_json::from_value(value.clone()).unwrap();
    let mut successor = value.clone();
    successor["publication"] = serde_json::json!({"Preserve": {"predecessor": receipt.commit_id}});
    let correct: CanonicalRequiredOutputOrigin = serde_json::from_value(successor.clone()).unwrap();
    let mut wrong = successor.clone();
    wrong["publication"] = serde_json::json!({"Preserve": {"predecessor": crate::history::data::CommitId(receipt.commit_id.0 + 1)}});
    let wrong: CanonicalRequiredOutputOrigin = serde_json::from_value(wrong).unwrap();
    let mut extra = successor;
    extra["meaning"]["family"] = serde_json::json!("another-owner");
    let extra: CanonicalRequiredOutputOrigin = serde_json::from_value(extra).unwrap();
    let none = || std::iter::empty::<&CanonicalRequiredOutputOrigin>();
    let one = std::iter::once;
    exact(one(&expected), none(), one(&correct), true, context).unwrap();
    assert!(exact(one(&expected), none(), none(), true, context).is_err());
    assert!(exact(one(&expected), none(), [&correct, &extra].into_iter(), true, context).is_err());
    assert!(exact(one(&expected), none(), one(&wrong), true, context).is_err());
    // A later stage's survivor (public #348) may be absent, or recorded from
    // its exact predecessor as older records list it; nothing else passes.
    exact(none(), one(&expected), none(), true, context).unwrap();
    exact(none(), one(&expected), one(&correct), true, context).unwrap();
    assert!(exact(none(), one(&expected), one(&wrong), true, context).is_err());
    assert!(exact(none(), one(&expected), one(&extra), true, context).is_err());
}
