//! Pure exact-owner assertions, borrowing the existing codec unit host's context.
use super::*;
use crate::mvcc::{RequiredOutputProducingImplementation, RequiredOutputPublicationMeaning};
pub(crate) fn assert_exact_owner_refusals(
    original: Arc<CanonicalCommitEnvelope>,
    basis: &crate::branch::RelationalBranchBasisDescriptor,
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
        "commit": original.commit, "basis": basis,
        "producer": "owner-selection-data", "source_scope_owned": true,
        "dependency_identity": worth_foundational::CanonicalDigestId::new([5;32]),
        "outputs": [], "relations": [], "meaning": meaning, "publication": "Birth",
        "birth": original.commit.commit_id, "birth_producer": "owner-selection-data",
        "birth_dependency": worth_foundational::CanonicalDigestId::new([6;32]), "members": []
    });
    let expected = SelectedOutputPublication {
        evidence: crate::publication::ownership::PublicationEvidence::Canonical(original.clone()),
        origin: Arc::new(serde_json::from_value(value.clone()).unwrap()),
        ordinal: 0,
    };
    let mut successor = value.clone();
    successor["publication"] =
        serde_json::json!({"Preserve": {"predecessor": original.commit.commit_id}});
    let correct: CanonicalRequiredOutputOrigin = serde_json::from_value(successor.clone()).unwrap();
    let mut wrong = successor.clone();
    wrong["publication"] = serde_json::json!({"Preserve": {"predecessor": crate::history::data::CommitId(original.commit.commit_id.0 + 1)}});
    let wrong: CanonicalRequiredOutputOrigin = serde_json::from_value(wrong).unwrap();
    let mut extra = successor;
    extra["meaning"]["family"] = serde_json::json!("another-owner");
    let extra: CanonicalRequiredOutputOrigin = serde_json::from_value(extra).unwrap();
    exact(
        std::slice::from_ref(&expected),
        std::iter::once(&correct),
        true,
        context,
    )
    .unwrap();
    assert!(exact(
        std::slice::from_ref(&expected),
        std::iter::empty(),
        true,
        context
    )
    .is_err());
    assert!(exact(
        std::slice::from_ref(&expected),
        [&correct, &extra].into_iter(),
        true,
        context
    )
    .is_err());
    assert!(exact(
        std::slice::from_ref(&expected),
        std::iter::once(&wrong),
        true,
        context
    )
    .is_err());
}
